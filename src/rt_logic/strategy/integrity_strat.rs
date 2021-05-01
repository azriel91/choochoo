use std::{future::Future, marker::PhantomData, pin::Pin};

use daggy::{
    petgraph::{graph::DefaultIx, visit::IntoNodeIdentifiers},
    NodeIndex,
};
use futures::{stream, stream::StreamExt};
use indicatif::ProgressStyle;

use crate::{
    rt_logic::VisitStatusUpdater,
    rt_model::{Destination, Station, VisitStatus},
};

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
        F: for<'a> Fn(&'a R, &'a mut Station<E>) -> Pin<Box<dyn Future<Output = &'a R> + 'a>>,
    {
        let node_ids = dest
            .stations
            .node_identifiers()
            .collect::<Vec<NodeIndex<DefaultIx>>>();
        let mut node_ids_queued = Vec::<NodeIndex<DefaultIx>>::with_capacity(node_ids.len());
        let visit_logic = &visit_logic;

        loop {
            let frozen = dest.stations.frozen();
            node_ids.iter().for_each(|node_id| {
                let station = &frozen[*node_id];
                if !station.progress_bar.is_finished() {
                    match station.visit_status {
                        VisitStatus::NotReady => {}
                        VisitStatus::ParentFail => {
                            let progress_style = ProgressStyle::default_bar()
                                .template(Station::<E>::STYLE_PARENT_FAILED);
                            station.progress_bar.set_style(progress_style);
                            station.progress_bar.abandon();
                        }
                        VisitStatus::Queued => {}
                        VisitStatus::InProgress => {}
                        VisitStatus::VisitSuccess => {
                            let progress_style = ProgressStyle::default_bar()
                                .template(Station::<E>::STYLE_SUCCESS_BYTES);
                            station.progress_bar.set_style(progress_style);
                            station.progress_bar.finish();
                        }
                        VisitStatus::VisitUnnecessary => {
                            let progress_style = ProgressStyle::default_bar()
                                .template(Station::<E>::STYLE_UNCHANGED_BYTES);
                            station.progress_bar.set_style(progress_style);
                            station.progress_bar.finish();
                        }
                        VisitStatus::VisitFail => {
                            let progress_style =
                                ProgressStyle::default_bar().template(Station::<E>::STYLE_FAILED);
                            station.progress_bar.set_style(progress_style);
                            station.progress_bar.abandon();
                        }
                    }
                } else {
                    match station.visit_status {
                        VisitStatus::NotReady | VisitStatus::Queued => {
                            let progress_style =
                                ProgressStyle::default_bar().template(Station::<E>::STYLE_QUEUED);
                            station.progress_bar.set_style(progress_style);
                        }
                        VisitStatus::InProgress
                        | VisitStatus::ParentFail
                        | VisitStatus::VisitSuccess
                        | VisitStatus::VisitUnnecessary
                        | VisitStatus::VisitFail => {}
                    }
                }
                if station.visit_status == VisitStatus::Queued {
                    node_ids_queued.push(*node_id);
                }
            });

            if !node_ids_queued.is_empty() {
                let seed_ref = &seed;
                stream::iter(
                    dest.stations
                        .node_weights_mut()
                        .filter(|station| station.visit_status == VisitStatus::Queued),
                )
                .for_each_concurrent(4, |station| async move {
                    let progress_style =
                        ProgressStyle::default_bar().template(Station::<E>::STYLE_IN_PROGRESS);
                    station.progress_bar.set_style(progress_style);
                    visit_logic(seed_ref, station).await;
                })
                .await;
            } else {
                break;
            }

            VisitStatusUpdater::update(&mut dest.stations);

            node_ids_queued.clear();
        }

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
        cfg_model::{StationFn, StationId, StationIdInvalidFmt, StationSpec, StationSpecFns},
        rt_model::{Destination, Station, Stations, VisitStatus},
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
            add_station(&mut stations, "a", VisitStatus::VisitSuccess, Ok((tx, 0)))?;
            add_station(&mut stations, "b", VisitStatus::VisitFail, Err(()))?;
            add_station(&mut stations, "c", VisitStatus::ParentFail, Err(()))?;
            Destination { stations }
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
            add_station(&mut stations, "a", VisitStatus::Queued, Ok((tx.clone(), 0)))?;
            add_station(&mut stations, "b", VisitStatus::Queued, Ok((tx.clone(), 1)))?;
            add_station(&mut stations, "c", VisitStatus::NotReady, Ok((tx, 2)))?;
            Destination { stations }
        };

        let (call_count, stations_sequence) = call_iter(dest, Some(rx))?;

        assert_eq!(3, call_count);
        assert_eq!(vec![0u8, 1u8, 2u8], stations_sequence);
        Ok(())
    }

    fn add_station(
        stations: &mut Stations<()>,
        station_id: &'static str,
        visit_status: VisitStatus,
        visit_result: Result<(Sender<u8>, u8), ()>,
    ) -> Result<(), StationIdInvalidFmt<'static>> {
        let station_spec_fns = {
            let visit_fn = match visit_result {
                Ok((tx, n)) => StationFn::new(move |station, _| {
                    let tx = tx.clone();
                    Box::pin(async move {
                        station.visit_status = VisitStatus::VisitSuccess;
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
        let station = Station::new(station_spec, visit_status);
        stations.add_node(station);
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
            let resources = IntegrityStrat::iter(&mut dest, resources, |resources, station| {
                Box::pin(async move {
                    station
                        .visit(&Resources::default())
                        .await
                        .expect("Failed to visit station.");

                    *resources.borrow_mut::<u32>() += 1;
                    resources
                })
            })
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
