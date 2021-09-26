use std::{future::Future, marker::PhantomData, pin::Pin};

use choochoo_cfg_model::{indicatif::ProgressStyle, StationProgress};
use choochoo_rt_model::{Error, StationMut, StationRtId};
use futures::{
    stream,
    stream::{StreamExt, TryStreamExt},
};
use tokio::sync::mpsc::{Receiver, Sender};

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
        seed: &R,
        visit_logic: &F,
        mut stations_queued_rx: Receiver<StationMut<'f, E>>,
        stations_done_tx: Sender<StationRtId>,
    ) -> Result<(), Error<E>>
    where
        F: for<'a, 'station> Fn(
            &'a mut StationMut<'station, E>,
            &'a R,
        ) -> Pin<Box<dyn Future<Output = &'a R> + 'a>>,
    {
        let stations_done_tx_ref = &stations_done_tx;

        stream::poll_fn(|context| stations_queued_rx.poll_recv(context))
            .map(Result::<_, Error<E>>::Ok)
            .try_for_each_concurrent(4, |mut station| async move {
                let progress_style =
                    ProgressStyle::default_bar().template(StationProgress::STYLE_IN_PROGRESS_BYTES);
                station.progress.progress_bar.set_style(progress_style);

                visit_logic(&mut station, seed).await;

                stations_done_tx_ref
                    .send(station.rt_id)
                    .await
                    .map_err(|_error| Error::StationVisitNotify {
                        station_spec: station.spec.clone(),
                    })?;
                Ok(())
            })
            .await?;

        stations_queued_rx.close();

        Ok(())
    }
}
