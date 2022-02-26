use std::fmt;

use choochoo_cfg_model::rt::{ResourceIds, TrainResources};

/// Record of what happened during a train's drive.
#[derive(Debug)]
pub struct TrainReport<E> {
    /// Record of what happened during a train's drive.
    train_resources: TrainResources<E>,
    /// Resource IDs produced by visiting each station.
    resource_ids: ResourceIds,
}

impl<E> TrainReport<E>
where
    E: fmt::Debug + Send + Sync + 'static,
{
    /// Returns a new TrainReport.
    pub fn new(train_resources: TrainResources<E>, resource_ids: ResourceIds) -> Self {
        Self {
            train_resources,
            resource_ids,
        }
    }

    /// Record of what happened during a train's drive.
    pub fn train_resources(&self) -> &TrainResources<E> {
        &self.train_resources
    }

    /// Resource IDs produced by visiting each station.
    pub fn resource_ids(&self) -> &ResourceIds {
        &self.resource_ids
    }
}

impl<E> Default for TrainReport<E>
where
    E: fmt::Debug + Send + Sync + 'static,
{
    fn default() -> Self {
        Self {
            train_resources: TrainResources::<E>::new(),
            resource_ids: ResourceIds::default(),
        }
    }
}
