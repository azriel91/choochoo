use std::{future::Future, marker::PhantomData};

use futures::stream::{self, StreamExt};

use crate::{
    rt_logic::VisitStatusUpdater,
    rt_model::{Destination, Station},
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
/// `Item` return type needs to be:
///
/// ```rust
/// type Item<'a> = &'a mut Station<E>;
/// ```
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
    /// # Parameters
    ///
    /// * `dest`: `Destination` whose stations to visit.
    pub async fn iter<F, Fut, R>(
        dest: &mut Destination<E>,
        mut return_seed: R,
        mut visit_logic: F,
    ) -> R
    where
        F: FnMut(R, &mut Station<E>) -> Fut,
        Fut: Future<Output = R>,
    {
        loop {
            match dest.stations_queued() {
                None => break,
                Some(stations_queued) => {
                    return_seed = stream::iter(stations_queued)
                        .fold(return_seed, &mut visit_logic)
                        .await;
                }
            }
            VisitStatusUpdater::update(&mut dest.stations);
        }

        return_seed
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
        cfg_model::{StationSpec, VisitFn},
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
            add_station(&mut stations, VisitStatus::VisitSuccess, Ok((tx, 0)));
            add_station(&mut stations, VisitStatus::VisitFail, Err(()));
            add_station(&mut stations, VisitStatus::ParentFail, Err(()));
            Destination { stations }
        };

        let (call_count, stations_sequence) = call_iter(dest, None)?;

        assert_eq!(0, call_count);
        assert!(stations_sequence.is_empty());
        Ok(())
    }

    #[test]
    fn returns_queued_stations() -> Result<(), Box<dyn std::error::Error>> {
        let (tx, rx) = mpsc::channel(10);
        let dest = {
            let mut stations = Stations::new();
            add_station(&mut stations, VisitStatus::Queued, Ok((tx.clone(), 0)));
            add_station(&mut stations, VisitStatus::Queued, Ok((tx.clone(), 1)));
            add_station(&mut stations, VisitStatus::NotReady, Ok((tx, 2)));
            Destination { stations }
        };

        let (call_count, stations_sequence) = call_iter(dest, Some(rx))?;

        assert_eq!(2, call_count);
        assert_eq!(vec![0u8, 1u8], stations_sequence);
        Ok(())
    }

    #[test]
    fn returns_propagated_newly_queued_stations() -> Result<(), Box<dyn std::error::Error>> {
        let (tx, rx) = mpsc::channel(10);
        let dest = {
            let mut stations = Stations::new();
            add_station(&mut stations, VisitStatus::Queued, Ok((tx.clone(), 0)));
            add_station(&mut stations, VisitStatus::Queued, Ok((tx.clone(), 1)));
            add_station(&mut stations, VisitStatus::NotReady, Ok((tx, 2)));
            Destination { stations }
        };

        let (call_count, stations_sequence) = call_iter(dest, Some(rx))?;

        assert_eq!(3, call_count);
        assert_eq!(vec![0u8, 1u8, 2u8], stations_sequence);
        Ok(())
    }

    fn add_station(
        stations: &mut Stations<()>,
        visit_status: VisitStatus,
        visit_result: Result<(Sender<u8>, u8), ()>,
    ) {
        let visit_fn = match visit_result {
            Ok((tx, n)) => VisitFn::new(move |station| {
                let tx = tx.clone();
                Box::pin(async move {
                    station.visit_status = VisitStatus::VisitSuccess;
                    tx.send(n).await.map_err(|_| ())
                })
            }),
            _ => VisitFn::new(|_station| Box::pin(async move { Result::<(), ()>::Err(()) })),
        };
        let station_spec = StationSpec::new(visit_fn);
        let station = Station::new(station_spec, visit_status);
        stations.add_node(station);
    }

    fn call_iter(
        mut dest: Destination<()>,
        rx: Option<Receiver<u8>>,
    ) -> Result<(u32, Vec<u8>), Box<dyn std::error::Error>> {
        let rt = runtime::Builder::new_current_thread().build()?;
        let call_count_and_values = rt.block_on(async {
            let call_count = IntegrityStrat::iter(&mut dest, 0, |call_count, station| async move {
                station.visit().await.expect("Failed to visit station.");

                call_count + 1
            })
            .await;

            let mut received_values = Vec::new();
            if let Some(mut rx) = rx {
                while let Some(n) = rx.recv().await {
                    received_values.push(n);
                }
            }

            (call_count, received_values)
        });

        Ok(call_count_and_values)
    }
}
