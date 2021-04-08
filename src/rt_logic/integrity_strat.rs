use std::{future::Future, marker::PhantomData, pin::Pin};

use daggy::{
    petgraph::{graph::DefaultIx, visit::IntoNodeIdentifiers},
    NodeIndex,
};

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
    pub async fn iter<F, R>(dest: &mut Destination<E>, mut seed: R, mut visit_logic: F) -> R
    where
        F: for<'a> FnMut(
            R,
            NodeIndex<DefaultIx>,
            &'a mut Station<E>,
        ) -> Pin<Box<dyn Future<Output = R> + Send + Sync + 'a>>,
    {
        let node_ids = dest
            .stations
            .node_identifiers()
            .collect::<Vec<NodeIndex<DefaultIx>>>();
        let mut node_ids_queued = Vec::<NodeIndex<DefaultIx>>::with_capacity(node_ids.len());
        let visit_logic = &mut visit_logic;

        loop {
            let mut frozen = dest.stations.frozen();
            node_ids.iter().for_each(|node_id| {
                let station = &frozen[*node_id];
                if station.visit_status == VisitStatus::Queued {
                    node_ids_queued.push(*node_id);
                }
            });

            if !node_ids_queued.is_empty() {
                for node_id in node_ids_queued.iter() {
                    let node_id = *node_id;
                    let station = &mut frozen[node_id];
                    seed = visit_logic(seed, node_id, station).await;
                }
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
    use tokio::{
        runtime,
        sync::mpsc::{self, Receiver, Sender},
    };

    use super::IntegrityStrat;
    use crate::{
        cfg_model::{StationFn, StationId, StationIdInvalidFmt, StationSpec},
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
        let visit_fn = match visit_result {
            Ok((tx, n)) => StationFn::new(move |station| {
                let tx = tx.clone();
                Box::pin(async move {
                    station.visit_status = VisitStatus::VisitSuccess;
                    tx.send(n).await.map_err(|_| ())
                })
            }),
            _ => StationFn::new(|_station| Box::pin(async move { Result::<(), ()>::Err(()) })),
        };
        let name = String::from(station_id);
        let station_id = StationId::new(station_id)?;
        let station_spec = StationSpec::new(station_id, name, String::from(""), visit_fn);
        let station = Station::new(station_spec, visit_status);
        stations.add_node(station);
        Ok(())
    }

    fn call_iter(
        mut dest: Destination<()>,
        rx: Option<Receiver<u8>>,
    ) -> Result<(u32, Vec<u8>), Box<dyn std::error::Error>> {
        let rt = runtime::Builder::new_current_thread().build()?;
        let call_count_and_values = rt.block_on(async {
            let call_count = IntegrityStrat::iter(&mut dest, 0, |call_count, _, station| {
                Box::pin(async move {
                    station.visit().await.expect("Failed to visit station.");

                    call_count + 1
                })
            })
            .await;

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
