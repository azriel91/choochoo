use std::marker::PhantomData;

use futures::{stream, stream::StreamExt, TryStreamExt};
use indicatif::ProgressStyle;
use rt_map::RefMut;
use tokio::sync::mpsc::{error::SendError, Receiver, Sender};

use crate::{
    cfg_model::{StationProgress, StationSpec},
    rt_logic::VisitStatusUpdater,
    rt_model::{Destination, StationRtId, VisitStatus},
};

/// Listens to station visit completions and queues more stations.
#[derive(Debug)]
pub(crate) struct StationQueuer<E>(PhantomData<E>);

impl<E> StationQueuer<E> {
    /// Listens to station visit completions and queues more stations.
    ///
    /// # Parameters
    ///
    /// * `dest`: `Destination` whose stations to visit.
    /// * `stations_queued_tx`: Sender to write queued stations to.
    /// * `stations_done_rx`: Receiver for station that have been visited.
    pub async fn run<'f>(
        dest: &'f Destination<E>,
        stations_queued_tx: Sender<(
            &'f StationSpec<E>,
            StationRtId,
            RefMut<'f, StationProgress<E>>,
        )>,
        mut stations_done_rx: Receiver<StationRtId>,
    ) {
        let stations_queued_tx_ref = &stations_queued_tx;
        let mut stations_in_progress_count = 0;

        loop {
            stations_in_progress_count = stream::iter(Self::stations_queued(dest))
                .map(Result::<_, SendError<_>>::Ok)
                .try_fold(
                    stations_in_progress_count,
                    |stations_in_progress_count, station_and_progress| async move {
                        stations_queued_tx_ref.send(station_and_progress).await?;

                        Ok(stations_in_progress_count + 1)
                    },
                )
                .await
                .unwrap_or_else(|e| panic!("Failed to queue additional station. {}", e)); // TODO: properly propagate this.

            if stations_in_progress_count == 0 {
                stations_done_rx.close();
                break;
            } else {
                // TODO: will this wait indefinitely?
                if let Some(station_rt_id) = stations_done_rx.recv().await {
                    stations_in_progress_count -= 1;
                    VisitStatusUpdater::update_children(dest, station_rt_id);

                    dest.station_progresses()
                        .values()
                        .for_each(|station_progress| {
                            if let Some(station_progress) = station_progress.try_borrow() {
                                Self::station_progress_bar_update(&station_progress)
                            }
                        });
                } else {
                    // No more stations will be sent.
                    stations_done_rx.close();
                    drop(stations_queued_tx);
                    break;
                }
            }
        }
    }

    fn stations_queued(
        dest: &Destination<E>,
    ) -> impl Iterator<
        Item = (
            &StationSpec<E>,
            StationRtId,
            rt_map::RefMut<StationProgress<E>>,
        ),
    > + '_ {
        dest.stations().iter().filter_map(move |station_spec| {
            let station_rt_id = dest.station_id_to_rt_id().get(station_spec.id()).copied();
            if let Some(station_rt_id) = station_rt_id {
                let station_progress = dest.station_progresses().try_borrow_mut(&station_rt_id);
                if let Some(station_progress) = station_progress {
                    if station_progress.visit_status == VisitStatus::Queued {
                        Some((station_spec, station_rt_id, station_progress))
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            }
        })
    }

    fn station_progress_bar_update(station_progress: &StationProgress<E>) {
        if !station_progress.progress_bar.is_finished() {
            match station_progress.visit_status {
                VisitStatus::NotReady | VisitStatus::Queued => {
                    let progress_style =
                        ProgressStyle::default_bar().template(StationProgress::<E>::STYLE_QUEUED);
                    station_progress.progress_bar.set_style(progress_style);
                }
                VisitStatus::ParentFail => {
                    let progress_style = ProgressStyle::default_bar()
                        .template(StationProgress::<E>::STYLE_PARENT_FAILED);
                    station_progress.progress_bar.set_style(progress_style);
                    station_progress.progress_bar.abandon();
                }
                VisitStatus::InProgress => {}
                VisitStatus::VisitSuccess => {
                    let progress_style = ProgressStyle::default_bar()
                        .template(StationProgress::<E>::STYLE_SUCCESS_BYTES);
                    station_progress.progress_bar.set_style(progress_style);
                    station_progress.progress_bar.finish();
                }
                VisitStatus::VisitUnnecessary => {
                    let progress_style = ProgressStyle::default_bar()
                        .template(StationProgress::<E>::STYLE_UNCHANGED_BYTES);
                    station_progress.progress_bar.set_style(progress_style);
                    station_progress.progress_bar.finish();
                }
                VisitStatus::CheckFail | VisitStatus::VisitFail => {
                    let progress_style =
                        ProgressStyle::default_bar().template(StationProgress::<E>::STYLE_FAILED);
                    station_progress.progress_bar.set_style(progress_style);
                    station_progress.progress_bar.abandon();
                }
            }
        }
    }
}
