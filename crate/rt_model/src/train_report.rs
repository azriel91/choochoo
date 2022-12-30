use std::fmt;

use choochoo_cfg_model::rt::{ResIds, TrainResources};

/// Record of what happened during a train's drive.
#[derive(Debug)]
pub struct TrainReport<E> {
    /// Record of what happened during a train's drive.
    train_resources: TrainResources<E>,
    /// Resource IDs produced by visiting each station.
    res_ids: ResIds,
}

impl<E> TrainReport<E>
where
    E: fmt::Debug + Send + Sync + 'static,
{
    /// Returns a new TrainReport.
    pub fn new(train_resources: TrainResources<E>, res_ids: ResIds) -> Self {
        Self {
            train_resources,
            res_ids,
        }
    }

    /// Record of what happened during a train's drive.
    pub fn train_resources(&self) -> &TrainResources<E> {
        &self.train_resources
    }

    /// Resource IDs produced by visiting each station.
    pub fn res_ids(&self) -> &ResIds {
        &self.res_ids
    }
}

impl<E> Default for TrainReport<E>
where
    E: fmt::Debug + Send + Sync + 'static,
{
    fn default() -> Self {
        Self {
            train_resources: TrainResources::<E>::new(),
            res_ids: ResIds::default(),
        }
    }
}
