pub mod config;
pub mod overseer;
pub mod registry;

pub mod service;

pub mod models;

/// A const address to bind and send messages to
pub const NODEMANAGER_ADDR: &str = "_internal.nodemanager";

/// A const address for the Overseer worker to send messages to
pub const OVERSEER_ADDR: &str = "_internal.overseer";

/// The main node-manager service running on remote nodes
pub use service::{IdentityOverride, NodeManager, NodeManagerWorker};
