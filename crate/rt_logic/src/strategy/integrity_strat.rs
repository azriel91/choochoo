use std::{future::Future, marker::PhantomData, pin::Pin};

use choochoo_cfg_model::rt::{StationMut, VisitStatus};
use choochoo_rt_model::{Destination, Error};
use tokio::sync::mpsc;

use self::{station_queuer::StationQueuer, station_visitor::StationVisitor};

mod station_queuer;
mod station_visitor;

/// [`StationMut`]s to process with integrity guarantees.
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
/// <https://users.rust-lang.org/t/how-to-implement-iterator-where-next-elements-depends-on-previous-elements-mutation/54209>
///
/// [`Stream`]: futures::stream::Stream
#[derive(Debug)]
pub struct IntegrityStrat<E> {
    /// Marker.
    marker: PhantomData<E>,
}

impl<E> IntegrityStrat<E> {
    /// Maximum number of stations to visit concurrently.
    const QUEUE_LIMIT: usize = 8;

    /// Runs the visit logic over each station with integrity guarantees.
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
    /// * <https://users.rust-lang.org/t/function-that-takes-a-closure-with-mutable-reference-that-returns-a-future/54324>
    /// * <https://github.com/rust-lang/rust/issues/74497#issuecomment-661995588>
    ///
    /// # Parameters
    ///
    /// * `dest`: `Destination` whose stations to visit.
    /// * `seed`: Initial seed for the return value.
    /// * `visit_logic`: Logic to run to visit a `Station`.
    pub async fn iter<F, R>(dest: &Destination<E>, seed: R, visit_logic: F) -> Result<R, Error<E>>
    where
        F: for<'a, 'station> Fn(
            &'a mut StationMut<'station, E>,
            &'a R,
        ) -> Pin<Box<dyn Future<Output = &'a R> + 'a>>,
    {
        let (stations_queued_tx, stations_queued_rx) = mpsc::channel(64);
        let (stations_done_tx, stations_done_rx) = mpsc::channel(64);

        // Listen to completion and queue more stations task.
        let station_queuer = StationQueuer::run(
            dest,
            stations_queued_tx,
            stations_done_rx,
            VisitStatus::VisitQueued,
            Self::QUEUE_LIMIT,
        );

        // Process task.
        let station_visitor =
            StationVisitor::visit(&seed, &visit_logic, stations_queued_rx, stations_done_tx);

        futures::try_join!(station_queuer, station_visitor)?;

        Ok(seed)
    }

    pub async fn iter_sequential<F, R>(
        dest: &Destination<E>,
        mut seed: R,
        visit_logic: F,
    ) -> Result<R, Error<E>>
    where
        F: for<'a, 'station> Fn(
            &'a mut StationMut<'station, E>,
            &'a mut R,
        ) -> Pin<Box<dyn Future<Output = ()> + 'a>>,
    {
        let (stations_queued_tx, stations_queued_rx) = mpsc::channel(64);
        let (stations_done_tx, stations_done_rx) = mpsc::channel(64);

        // Listen to completion and queue more stations task.
        let station_queuer = StationQueuer::run(
            dest,
            stations_queued_tx,
            stations_done_rx,
            VisitStatus::SetupQueued,
            1, // Limit to 1 station between iterations
        );

        // Process task.
        let station_visitor = StationVisitor::visit_sequential(
            &mut seed,
            &visit_logic,
            stations_queued_rx,
            stations_done_tx,
        );

        futures::try_join!(station_queuer, station_visitor)?;

        Ok(seed)
    }
}
