use std::{future::Future, marker::PhantomData, pin::Pin};

use futures::{stream, stream::StreamExt};
use indicatif::ProgressStyle;
use tokio::sync::mpsc::{Receiver, Sender};

use crate::{
    cfg_model::StationProgress,
    rt_model::{Destination, Station, StationRtId},
};

/// Listens for queued station and visits them.
#[derive(Debug)]
pub(crate) struct StationVisitor<E>(PhantomData<E>);

impl<E> StationVisitor<E> {
    /// Listens for queued station and visits them.
    ///
    /// # Parameters
    ///
    /// * `seed`: Initial seed for the return value.
    /// * `visit_logic`: Logic to run to visit a `Station`.
    /// * `stations_queued_rx`: Receiver for queued stations.
    /// * `stations_done_tx`: Sender to write visited stations to.
    pub async fn visit<'f, F, R>(
        dest: &Destination<E>,
        visit_logic: &F,
        seed: &R,
        mut stations_queued_rx: Receiver<Station<'f, E>>,
        stations_done_tx: Sender<StationRtId>,
    ) where
        F: for<'a, 'station> Fn(
            &'a Destination<E>,
            &'a mut Station<'station, E>,
            &'a R,
        ) -> Pin<Box<dyn Future<Output = &'a R> + 'a>>,
    {
        let stations_done_tx_ref = &stations_done_tx;
        stream::poll_fn(|context| stations_queued_rx.poll_recv(context))
            .for_each_concurrent(4, |mut station| async move {
                let progress_style = ProgressStyle::default_bar()
                    .template(StationProgress::<E>::STYLE_IN_PROGRESS_BYTES);
                station.progress.progress_bar.set_style(progress_style);

                visit_logic(dest, &mut station, seed).await;

                stations_done_tx_ref
                    .send(station.rt_id)
                    .await
                    .unwrap_or_else(|e| {
                        panic!(
                            "Failed to notify that `{}` is completed. {}",
                            station.spec.id(),
                            e
                        )
                    });
            })
            .await;
        stations_queued_rx.close();
    }
}
