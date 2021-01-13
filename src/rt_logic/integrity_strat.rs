use std::{
    fmt::{self, Debug},
    iter::Filter,
    marker::PhantomData,
    pin::Pin,
};

use daggy::{petgraph::graph::DefaultIx, NodeWeightsMut};
use futures::{
    task::{Context, Poll},
    Stream,
};

use crate::rt_model::{Destination, Station};

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
#[derive(Debug)]
pub struct IntegrityStrat<'dest, E> {
    /// Destination whose stations to visit.
    dest: &'dest mut Destination<E>,
}

impl<'dest, E> IntegrityStrat<'dest, E> {
    /// Returns a stream of [`Station`]s to process with integrity guarantees.
    ///
    /// See the [`IntegrityStrat`] type level documentation for more details.
    ///
    /// # Parameters
    ///
    /// * `dest`: `Destination` whose stations to visit.
    pub fn iter<'f>(dest: &'f mut Destination<E>) -> IntegrityStratIter<'f, E> {
        IntegrityStratIter {
            stations: dest.stations_queued(),
            marker: PhantomData,
        }
    }
}

pub struct IntegrityStratIter<'dest, E> {
    /// Iterator of stations to visit.
    stations: Filter<
        NodeWeightsMut<'dest, Station<E>, DefaultIx>,
        for<'f> fn(&'f &'dest mut Station<E>) -> bool,
    >,
    /// Marker.
    marker: PhantomData<&'dest E>,
}

impl<'dest, E> Debug for IntegrityStratIter<'dest, E>
where
    E: Debug,
{
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("Station")
            .field("stations", &"..")
            .finish()
    }
}

// See <https://users.rust-lang.org/t/returning-borrowed-values-from-an-iterator/1096> for implementation hint.
// See also <https://docs.rs/futures/0.3.9/futures/stream/trait.Stream.html>
impl<'dest, E> Stream for IntegrityStratIter<'dest, E> {
    type Item = &'dest mut Station<E>;

    // Read https://rust-lang.github.io/async-book/02_execution/03_wakeups.html to learn how to implement this.
    fn poll_next(
        mut self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Option<<Self as Stream>::Item>> {
        // Perhaps we need to keep a record of:
        //
        // * Node indices of stations that have not been processed.
        // * Edge indices where failures have occured.
        // * Edge indices that cannot be processed because of dependencies' failures.
        //
        // Not yet sure where to store it.

        if let Some(station) = self.stations.next() {
            Poll::Ready(Some(station))
        } else {
            Poll::Ready(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use futures::StreamExt;
    use tokio::{
        runtime,
        sync::mpsc::{self, Sender},
    };

    use super::IntegrityStrat;
    use crate::{
        cfg_model::{StationSpec, VisitFn},
        rt_model::{Destination, Station, Stations, VisitStatus},
    };

    #[test]
    fn returns_empty_stream_when_no_stations_exist() -> Result<(), Box<dyn std::error::Error>> {
        let rt = runtime::Builder::new_current_thread().build()?;
        let mut dest = Destination::<()>::default();

        let mut iterator = IntegrityStrat::iter(&mut dest);
        let station = rt.block_on(async move { iterator.next().await });

        assert_eq!(None, station);
        Ok(())
    }

    #[test]
    fn returns_empty_stream_when_station_all_visit_success_or_failed()
    -> Result<(), Box<dyn std::error::Error>> {
        let rt = runtime::Builder::new_current_thread().build()?;
        let (tx, _rx) = mpsc::channel(10);
        let mut dest = {
            let mut stations = Stations::new();
            add_station(&mut stations, VisitStatus::VisitSuccess, Ok((tx, 0)));
            add_station(&mut stations, VisitStatus::VisitFail, Err(()));
            add_station(&mut stations, VisitStatus::ParentFail, Err(()));
            Destination { stations }
        };

        let mut iterator = IntegrityStrat::iter(&mut dest);
        let station = rt.block_on(async move { iterator.next().await });

        assert_eq!(None, station);
        Ok(())
    }

    #[test]
    fn returns_queued_stations() -> Result<(), Box<dyn std::error::Error>> {
        let rt = runtime::Builder::new_current_thread().build()?;
        let (tx, mut rx) = mpsc::channel(10);
        let mut dest = {
            let mut stations = Stations::new();
            add_station(&mut stations, VisitStatus::Queued, Ok((tx.clone(), 0)));
            add_station(&mut stations, VisitStatus::Queued, Ok((tx.clone(), 1)));
            add_station(&mut stations, VisitStatus::NotReady, Ok((tx, 2)));
            Destination { stations }
        };

        let mut iterator = IntegrityStrat::iter(&mut dest);

        rt.block_on(async move {
            let station = iterator.next().await.expect("Expected station 0.");
            station.visit().await.expect("Failed to visit station 0.");
            assert_eq!(Some(0), rx.recv().await);

            let station = iterator.next().await.expect("Expected station 1.");
            station.visit().await.expect("Failed to visit station 1.");
            assert_eq!(Some(1), rx.recv().await);

            Ok(())
        })
    }

    fn add_station(
        stations: &mut Stations<()>,
        visit_status: VisitStatus,
        visit_result: Result<(Sender<u8>, u8), ()>,
    ) {
        let visit_fn = match visit_result {
            Ok((tx, n)) => VisitFn::new(move |_station| {
                let tx = tx.clone();
                Box::pin(async move { tx.send(n).await.map_err(|_| ()) })
            }),
            _ => VisitFn::new(|_station| Box::pin(async move { Result::<(), ()>::Err(()) })),
        };
        let station_spec = StationSpec::new(visit_fn);
        let station = Station::new(station_spec, visit_status);
        stations.add_node(station);
    }
}
