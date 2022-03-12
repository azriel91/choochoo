//! Runtime visit logic for the choochoo automation library.

pub use crate::{
    driver::Driver, op_status_updater::OpStatusUpdater, res_id_persister::ResIdPersister,
    resource_initializer::ResourceInitializer, train::Train,
};

mod driver;
mod op_status_updater;
mod res_id_persister;
mod resource_initializer;
mod train;
