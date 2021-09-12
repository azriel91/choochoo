use std::marker::PhantomData;

use choochoo_cfg_model::{indicatif::ProgressStyle, StationProgress, VisitStatus};
use choochoo_rt_model::{Destination, Error, StationMut, StationRtId};
use futures::{stream, stream::StreamExt, TryStreamExt};
use tokio::sync::mpsc::{Receiver, Sender};

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
        stations_queued_tx: Sender<StationMut<'f, E>>,
        mut stations_done_rx: Receiver<StationRtId>,
    ) -> Result<(), Error<E>> {
        let stations_queued_tx_ref = &stations_queued_tx;
        let mut stations_in_progress_count = 0;

        loop {
            stations_in_progress_count = stream::iter(Self::stations_queued(dest))
                .map(Result::<_, Error<E>>::Ok)
                .try_fold(
                    stations_in_progress_count,
                    |stations_in_progress_count, station| async move {
                        stations_queued_tx_ref
                            .send(station)
                            .await
                            .map_err(|error| {
                                let station_spec = (*error.0.spec).clone();
                                Error::StationQueue { station_spec }
                            })?;

                        Ok(stations_in_progress_count + 1)
                    },
                )
                .await?;

            if stations_in_progress_count == 0 {
                stations_done_rx.close();
                break;
            } else if let Some(_station_rt_id) = stations_done_rx.recv().await {
                stations_in_progress_count -= 1;

                // We have to update all progress bars, otherwise the multi progress bar will
                // interleave redraw operations, causing the output to be non-sensical, e.g. the
                // first few stations' progress bars are drawn multiple times, and the last few
                // stations' progress bars are not drawn at all.
                Self::progress_bar_update_all(dest);
            } else {
                // No more stations will be sent.
                stations_done_rx.close();
                break;
            }
        }

        Ok(())
    }

    fn stations_queued(dest: &Destination<E>) -> impl Iterator<Item = StationMut<'_, E>> + '_ {
        dest.stations_mut()
            .filter(move |station| station.progress.visit_status == VisitStatus::Queued)
    }

    fn progress_bar_update_all(dest: &Destination<E>) {
        dest.station_progresses()
            .values()
            .for_each(|station_progress| {
                if let Some(station_progress) = station_progress.try_borrow() {
                    Self::station_progress_bar_update(&station_progress)
                }
            });
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
