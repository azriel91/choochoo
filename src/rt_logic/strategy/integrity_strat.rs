use std::{future::Future, marker::PhantomData, pin::Pin};

use futures::{stream, stream::StreamExt};
use indicatif::ProgressStyle;
use tokio::sync::mpsc;

use crate::{
    cfg_model::{StationProgress, StationSpec},
    rt_logic::VisitStatusUpdater,
    rt_model::Destination,
};

use self::station_queuer::StationQueuer;

mod station_queuer;

/// [`Stream`] of [`Station`]s to process with integrity guarantees.
///
/// This means all stations are still checked and visited even though the
/// destination may already be reached at the beginning of the execution --
/// guaranteeing that if A depends on B, then when A is visited, we can be sure
/// that B has been successfully visited.
///
/// This strategy may do work even though the (final) end result may not need
/// some of that work to be accomplished. The benefit of this is compliance to
/// expected state, and reduces drift.
///
/// # Development Note
///
/// Originally this was intended to be implemented as a single [`Stream`], but
/// `Item` return type needs to be `type Item<'a> = &'a mut Station<E>;`
///
/// However, this requires GAT, which is not yet stable: <https://github.com/rust-lang/rust/issues/44265>.
/// See <https://users.rust-lang.org/t/returning-borrowed-values-from-an-iterator/1096> for implementation hint.
///
/// https://users.rust-lang.org/t/how-to-implement-iterator-where-next-elements-depends-on-previous-elements-mutation/54209
#[derive(Debug)]
pub struct IntegrityStrat<E> {
    /// Marker.
    marker: PhantomData<E>,
}

impl<E> IntegrityStrat<E> {
    /// Returns a stream of [`Station`]s to process with integrity guarantees.
    ///
    /// See the [`IntegrityStrat`] type level documentation for more details.
    ///
    /// # Implementation Note
    ///
    /// The `Pin<Box<_>>` around the `Fut` is required to allow this to compile.
    /// Without it, `&mut station` is only valid for the lifetime of the
    /// closure, not of the returned future.
    ///
    /// We need to make it valid for the lifetime of the returned future, but
    /// not so long that it extends beyond the `match` block.
    ///
    /// * https://users.rust-lang.org/t/function-that-takes-a-closure-with-mutable-reference-that-returns-a-future/54324
    /// * https://github.com/rust-lang/rust/issues/74497#issuecomment-661995588
    ///
    /// # Parameters
    ///
    /// * `dest`: `Destination` whose stations to visit.
    /// * `seed`: Initial seed for the return value.
    /// * `visit_logic`: Logic to run to visit a `Station`.
    pub async fn iter<F, R>(dest: &mut Destination<E>, seed: R, visit_logic: F) -> R
    where
        F: for<'a> Fn(
            &'a StationSpec<E>,
            &'a mut StationProgress<E>,
            &'a R,
        ) -> Pin<Box<dyn Future<Output = &'a R> + 'a>>,
    {
        let dest = &dest;
        let visit_logic = &visit_logic;
        let seed_ref = &seed;

        // Set `NotReady` stations to `Queued` if they have no dependencies.
        VisitStatusUpdater::update(dest);

        let (stations_queued_tx, mut stations_queued_rx) = mpsc::channel(64);
        let (stations_done_tx, stations_done_rx) = mpsc::channel(64);

        // Listen to completion and queue more stations task.
        let station_queuer = StationQueuer::run(dest, stations_queued_tx, stations_done_rx);

        // Process task.
        let station_visitor = async move {
            let stations_done_tx_ref = &stations_done_tx;
            stream::poll_fn(|context| stations_queued_rx.poll_recv(context))
                .for_each_concurrent(
                    4,
                    |(station_spec, station_rt_id, mut station_progress)| async move {
                        let progress_style = ProgressStyle::default_bar()
                            .template(StationProgress::<E>::STYLE_IN_PROGRESS_BYTES);
                        station_progress.progress_bar.set_style(progress_style);

                        visit_logic(station_spec, &mut station_progress, seed_ref).await;

                        stations_done_tx_ref
                            .send(station_rt_id)
                            .await
                            .unwrap_or_else(|e| {
                                panic!(
                                    "Failed to notify that `{}` is completed. {}",
                                    station_spec.id(),
                                    e
                                )
                            });
                    },
                )
                .await;
            stations_queued_rx.close();
            drop(stations_done_tx);
        };

        futures::join!(station_queuer, station_visitor);

        seed
    }
}

#[cfg(test)]
mod tests {
    use resman::Resources;
    use tokio::{
        runtime,
        sync::mpsc::{self, Receiver, Sender},
    };

    use super::IntegrityStrat;
    use crate::{
        cfg_model::{
            StationFn, StationId, StationIdInvalidFmt, StationProgress, StationSpec, StationSpecFns,
        },
        rt_model::{Destination, StationProgresses, Stations, VisitStatus},
    };

    #[test]
    fn returns_empty_stream_when_no_stations_exist() -> Result<(), Box<dyn std::error::Error>> {
        let dest = Destination::<()>::default();

        let (call_count, stations_sequence) = call_iter(dest, None)?;

        assert_eq!(0, call_count);
        assert!(stations_sequence.is_empty());
        Ok(())
    }

    #[test]
    fn returns_empty_stream_when_station_all_visit_success_or_failed()
    -> Result<(), Box<dyn std::error::Error>> {
        let (tx, _rx) = mpsc::channel(10);
        let dest = {
            let mut stations = Stations::new();
            let mut station_progresses = StationProgresses::new();
            add_station(
                &mut stations,
                &mut station_progresses,
                "a",
                VisitStatus::VisitSuccess,
                Ok((tx, 0)),
            )?;
            add_station(
                &mut stations,
                &mut station_progresses,
                "b",
                VisitStatus::VisitFail,
                Err(()),
            )?;
            add_station(
                &mut stations,
                &mut station_progresses,
                "c",
                VisitStatus::ParentFail,
                Err(()),
            )?;
            Destination::new(stations, station_progresses)
        };

        let (call_count, stations_sequence) = call_iter(dest, None)?;

        assert_eq!(0, call_count);
        assert!(stations_sequence.is_empty());
        Ok(())
    }

    #[test]
    fn returns_queued_stations_and_propagates_queued() -> Result<(), Box<dyn std::error::Error>> {
        let (tx, rx) = mpsc::channel(10);
        let dest = {
            let mut stations = Stations::new();
            let mut station_progresses = StationProgresses::new();
            add_station(
                &mut stations,
                &mut station_progresses,
                "a",
                VisitStatus::Queued,
                Ok((tx.clone(), 0)),
            )?;
            add_station(
                &mut stations,
                &mut station_progresses,
                "b",
                VisitStatus::Queued,
                Ok((tx.clone(), 1)),
            )?;
            add_station(
                &mut stations,
                &mut station_progresses,
                "c",
                VisitStatus::NotReady,
                Ok((tx, 2)),
            )?;
            Destination::new(stations, station_progresses)
        };

        let (_call_count, stations_sequence) = call_iter(dest, Some(rx))?;

        // assert_eq!(3, call_count);
        assert_eq!(vec![0u8, 1u8, 2u8], stations_sequence);
        Ok(())
    }

    fn add_station(
        stations: &mut Stations<()>,
        station_progresses: &mut StationProgresses<()>,
        station_id: &'static str,
        visit_status: VisitStatus,
        visit_result: Result<(Sender<u8>, u8), ()>,
    ) -> Result<(), StationIdInvalidFmt<'static>> {
        let station_spec_fns = {
            let visit_fn = match visit_result {
                Ok((tx, n)) => StationFn::new(move |station_progress, _| {
                    let tx = tx.clone();
                    Box::pin(async move {
                        station_progress.visit_status = VisitStatus::VisitSuccess;
                        tx.send(n).await.map_err(|_| ())
                    })
                }),
                Err(_) => StationFn::new(|_, _| Box::pin(async { Err(()) })),
            };
            StationSpecFns::new(visit_fn)
        };
        let name = String::from(station_id);
        let station_id = StationId::new(station_id)?;
        let station_spec = StationSpec::new(station_id, name, String::from(""), station_spec_fns);
        let station_progress = StationProgress::new(&station_spec, visit_status);
        let station_rt_id = stations.add_node(station_spec);
        station_progresses.insert(station_rt_id, station_progress);
        Ok(())
    }

    fn call_iter(
        mut dest: Destination<()>,
        rx: Option<Receiver<u8>>,
    ) -> Result<(u32, Vec<u8>), Box<dyn std::error::Error>> {
        let mut resources = Resources::default();
        resources.insert::<u32>(0);

        let rt = runtime::Builder::new_current_thread().build()?;
        let call_count_and_values = rt.block_on(async {
            let resources = IntegrityStrat::iter(
                &mut dest,
                resources,
                |station, station_progress, resources| {
                    Box::pin(async move {
                        station
                            .visit(station_progress, &Resources::default())
                            .await
                            .expect("Failed to visit station.");

                        *resources.borrow_mut::<u32>() += 1;
                        resources
                    })
                },
            )
            .await;
            let call_count = *resources.borrow::<u32>();

            let mut received_values = Vec::new();
            if let Some(mut rx) = rx {
                // Prevent test from hanging.
                rx.close();

                while let Some(n) = rx.recv().await {
                    received_values.push(n);
                }
            }

            (call_count, received_values)
        });

        Ok(call_count_and_values)
    }
}
