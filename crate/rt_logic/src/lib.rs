//! Runtime visit logic for the choochoo automation library.

pub use crate::{
    clean_driver::CleanDriver, create_driver::CreateDriver, op_status_updater::OpStatusUpdater,
    res_id_persister::ResIdPersister, resource_initializer::ResourceInitializer, train::Train,
};

mod clean_driver;
mod create_driver;
mod op_status_updater;
mod res_id_persister;
mod resource_initializer;
mod train;
