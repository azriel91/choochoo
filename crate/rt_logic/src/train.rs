use std::{fmt, marker::PhantomData, num::NonZeroUsize};

use choochoo_cfg_model::{
    indicatif::MultiProgress,
    rt::{OpStatus, ResIds, StationRtId, TrainResources, VisitOp},
};
use choochoo_rt_model::{error::StationSpecError, Destination, Error, TrainReport};
use futures::stream::{self, TryStreamExt};
use tokio::task::JoinHandle;

use crate::ResourceInitializer;

use self::{train_clean::TrainClean, train_create::TrainCreate};

mod train_clean;
mod train_create;

/// Ensures all carriages are at the destination.
#[derive(Debug)]
pub struct Train<E> {
    /// Maximum number of stations to run concurrently.
    concurrency_max: Option<NonZeroUsize>,
    /// Marker.
    marker: PhantomData<E>,
}

impl<E> Train<E>
where
    E: From<StationSpecError> + fmt::Debug + Send + Sync + 'static,
{
    /// Returns a `Train` to visit stations.
    ///
    /// The `concurrency_max` parameter is used as follows:
    ///
    /// * `None`: There is no concurrency limit.
    ///
    ///     This is the same as using [`Train::default`] to initialize a train.
    ///
    /// * `Some(n)`: At most `n` stations will be visited concurrently.
    ///
    ///     You may wish to use the [`std::thread::available_parallelism`]
    ///     function which  usually corresponds to the amount of CPUs or
    ///     processors a computer has.
    ///
    /// # Parameters
    ///
    /// * `concurrency_max`: Maximum number of stations to visit concurrently.
    fn new(concurrency_max: Option<NonZeroUsize>) -> Self {
        Self {
            concurrency_max,
            marker: PhantomData,
        }
    }

    /// Ensures the given destination is reached.
    pub async fn reach(
        &self,
        dest: &mut Destination<E>,
        visit_op: VisitOp,
    ) -> Result<TrainReport<E>, Error<E>> {
        let progress_fut = Self::progress_tracker_init(dest);

        if dest.station_specs().node_count() == 0 {
            Self::progress_tracker_join(dest, progress_fut).await?;
            return Ok(TrainReport::default());
        }

        let mut train_resources = TrainResources::new();
        ResourceInitializer::initialize(dest, &mut train_resources).await?;

        train_resources = Self::stations_setup(dest, visit_op, train_resources)
            .await
            .or_else(|error| {
                if let Error::StationSetup { train_resources } = error {
                    Ok(train_resources)
                } else {
                    Err(error)
                }
            })?;

        // If here are no errors during setup, then we visit each station.
        let train_report = if train_resources.station_errors().read().await.is_empty() {
            let train_report = match visit_op {
                VisitOp::Create => TrainCreate::stations_visit(self, dest, train_resources).await?,
                VisitOp::Clean => TrainClean::stations_visit(self, dest, train_resources).await?,
            };
            Self::progress_tracker_join(dest, progress_fut).await?;
            train_report
        } else {
            Self::progress_tracker_join(dest, progress_fut).await?;
            TrainReport::new(train_resources, ResIds::new())
        };

        Ok(train_report)
    }

    /// Initializes the progress tracker.
    fn progress_tracker_init(dest: &Destination<E>) -> JoinHandle<std::io::Result<()>> {
        let multi_progress = MultiProgress::new();
        dest.station_specs()
            .graph()
            .node_indices()
            .filter_map(|station_rt_id| dest.station_progresses().get(&station_rt_id))
            .for_each(|station_progress| {
                let progress_bar = station_progress.borrow().progress_bar().clone();
                let progress_bar_for_tick = progress_bar.clone();
                multi_progress.add(progress_bar);

                // Needed to render all progress bars.
                progress_bar_for_tick.tick();
            });

        tokio::task::spawn_blocking(move || multi_progress.join())
    }

    /// Waits for the progress tracker to complete.
    async fn progress_tracker_join(
        dest: &mut Destination<E>,
        progress_fut: JoinHandle<Result<(), std::io::Error>>,
    ) -> Result<(), Error<E>> {
        // We need to finish / abandon all progress bars, otherwise the `MultiProgress`
        // will never finish.
        dest.stations_mut().for_each(|station| {
            if !station.progress.progress_bar().is_finished() {
                station.progress.progress_bar().finish_at_current_pos();
            }
        });

        progress_fut
            .await
            .map_err(Error::MultiProgressTaskJoin)?
            .map_err(Error::MultiProgressJoin)?;

        Ok(())
    }

    async fn stations_setup(
        dest: &mut Destination<E>,
        visit_op: VisitOp,
        train_resources: TrainResources<E>,
    ) -> Result<TrainResources<E>, Error<E>> {
        match visit_op {
            VisitOp::Create => Self::stations_setup_create(dest, train_resources).await,
            VisitOp::Clean => Self::stations_setup_clean(dest, train_resources).await,
        }
    }

    async fn stations_setup_create(
        dest: &mut Destination<E>,
        train_resources: TrainResources<E>,
    ) -> Result<TrainResources<E>, Error<E>> {
        stream::iter(dest.stations_mut().map(Result::<_, Error<E>>::Ok))
            .try_fold(
                train_resources,
                |mut train_resources, mut station| async move {
                    let setup_result = station.create_setup(&mut train_resources).await;

                    match setup_result {
                        Ok(progress_limit) => {
                            station.progress.op_status = OpStatus::SetupSuccess;
                            station.progress.progress_limit_set(progress_limit);
                            station.progress.progress_style_update();
                            Ok(train_resources)
                        }
                        Err(station_error) => {
                            station.progress.op_status = OpStatus::SetupFail;
                            station.progress.progress_style_update();
                            Self::station_error_insert(
                                &train_resources,
                                station.rt_id,
                                station_error,
                            )
                            .await;
                            Err(Error::StationSetup { train_resources })
                        }
                    }
                },
            )
            .await
    }

    async fn stations_setup_clean(
        dest: &mut Destination<E>,
        train_resources: TrainResources<E>,
    ) -> Result<TrainResources<E>, Error<E>> {
        stream::iter(dest.stations_mut().map(Result::<_, Error<E>>::Ok))
            .try_fold(
                train_resources,
                |mut train_resources, mut station| async move {
                    let setup_result = station.clean_setup(&mut train_resources).await;

                    match setup_result {
                        Some(Ok(progress_limit)) => {
                            station.progress.op_status = OpStatus::SetupSuccess;
                            station.progress.progress_limit_set(progress_limit);
                            station.progress.progress_style_update();
                            Ok(train_resources)
                        }
                        Some(Err(station_error)) => {
                            station.progress.op_status = OpStatus::SetupFail;
                            station.progress.progress_style_update();
                            Self::station_error_insert(
                                &train_resources,
                                station.rt_id,
                                station_error,
                            )
                            .await;
                            Err(Error::StationSetup { train_resources })
                        }
                        None => {
                            station.progress.op_status = OpStatus::SetupSuccess;
                            Ok(train_resources)
                        }
                    }
                },
            )
            .await
    }

    async fn station_error_insert(
        train_resources: &TrainResources<E>,
        station_rt_id: StationRtId,
        station_error: E,
    ) {
        let station_errors = train_resources.station_errors();
        let mut station_errors = station_errors.write().await;
        station_errors.insert(station_rt_id, station_error);
    }
}

impl<E> Default for Train<E>
where
    E: From<StationSpecError> + fmt::Debug + Send + Sync + 'static,
{
    fn default() -> Self {
        Self::new(None)
    }
}
