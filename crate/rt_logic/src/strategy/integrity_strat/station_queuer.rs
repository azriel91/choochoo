use std::marker::PhantomData;

use choochoo_cfg_model::rt::{StationMut, StationProgress, StationRtId, VisitStatus};
use choochoo_rt_model::{Destination, Error};
use futures::{stream, stream::StreamExt, TryStreamExt};
use tokio::sync::mpsc::{Receiver, Sender};

use crate::VisitStatusUpdater;

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
        visit_status_queued: VisitStatus,
        queue_limit: usize,
    ) -> Result<(), Error<E>> {
        let stations_queued_tx_ref = &stations_queued_tx;
        let mut stations_in_progress_count = 0;

        loop {
            stations_in_progress_count = stream::iter(Self::stations_queued(
                dest,
                visit_status_queued,
                queue_limit,
            ))
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
            } else if let Some(station_rt_id) = stations_done_rx.recv().await {
                stations_in_progress_count -= 1;

                // Need to only call this after the `station` that is visited is dropped --
                // which is when its iteration completes. Otherwise the station's `VisitStatus`
                // will not be borrowable, and the station will be missed during calculation of
                // the child station's `VisitStatus`.
                VisitStatusUpdater::update_children(dest, station_rt_id);

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

    fn stations_queued(
        dest: &Destination<E>,
        visit_status_queued: VisitStatus,
        queue_limit: usize,
    ) -> impl Iterator<Item = StationMut<'_, E>> + '_ {
        dest.stations_mut()
            .filter(move |station| station.progress.visit_status == visit_status_queued)
            .take(queue_limit)
    }

    fn progress_bar_update_all(dest: &Destination<E>) {
        dest.station_progresses()
            .values()
            .for_each(|station_progress| {
                if let Ok(station_progress) = station_progress.try_borrow() {
                    Self::station_progress_bar_update(&station_progress)
                }
            });
    }

    fn station_progress_bar_update(station_progress: &StationProgress) {
        if !station_progress.progress_bar().is_finished() {
            station_progress.progress_style_update();
            match station_progress.visit_status {
                VisitStatus::SetupQueued
                | VisitStatus::SetupSuccess
                | VisitStatus::ParentPending
                | VisitStatus::VisitQueued
                | VisitStatus::InProgress => {}
                VisitStatus::SetupFail
                | VisitStatus::ParentFail
                | VisitStatus::CheckFail
                | VisitStatus::VisitFail => {
                    station_progress.progress_bar().abandon();
                }
                VisitStatus::VisitSuccess | VisitStatus::VisitUnnecessary => {
                    station_progress.progress_bar().finish();
                }
            }
        }
    }
}
