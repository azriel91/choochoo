use std::marker::PhantomData;

use crate::rt_model::Station;
use futures::{stream, stream::StreamExt, TryStreamExt};
use indicatif::ProgressStyle;
use tokio::sync::mpsc::{error::SendError, Receiver, Sender};

use crate::{
    cfg_model::StationProgress,
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
        stations_queued_tx: Sender<Station<'f, E>>,
        mut stations_done_rx: Receiver<StationRtId>,
    ) {
        let stations_queued_tx_ref = &stations_queued_tx;
        let mut stations_in_progress_count = 0;

        loop {
            stations_in_progress_count = stream::iter(Self::stations_queued(dest))
                .map(Result::<_, SendError<_>>::Ok)
                .try_fold(
                    stations_in_progress_count,
                    |stations_in_progress_count, station| async move {
                        stations_queued_tx_ref.send(station).await?;

                        Ok(stations_in_progress_count + 1)
                    },
                )
                .await
                .unwrap_or_else(|e| panic!("Failed to queue additional station. {}", e)); // TODO: properly propagate this.

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
    }

    fn stations_queued(dest: &Destination<E>) -> impl Iterator<Item = Station<'_, E>> + '_ {
        dest.stations().iter().filter_map(move |station_spec| {
            dest.station_id_to_rt_id()
                .get(station_spec.id())
                .and_then(|station_rt_id| {
                    dest.station_progresses()
                        .try_borrow_mut(station_rt_id)
                        .map(|station_progress| (*station_rt_id, station_progress))
                })
                .and_then(|(station_rt_id, station_progress)| {
                    if station_progress.visit_status == VisitStatus::Queued {
                        Some(Station {
                            spec: station_spec,
                            rt_id: station_rt_id,
                            progress: station_progress,
                        })
                    } else {
                        None
                    }
                })
        })
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
