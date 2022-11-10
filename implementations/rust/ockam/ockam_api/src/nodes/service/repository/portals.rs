use ockam::Context;
use ockam_node::tokio::sync::RwLockReadGuard;
use crate::nodes::{NodeManagerWorker, NodeManager, models::portal::{InletList, InletStatus, OutletList, OutletStatus}};

impl NodeManagerWorker {
    pub(crate) async fn build_inlets_list<'a>(
        &self,
        node_manager: &'a RwLockReadGuard<'_, NodeManager>,
        ctx: &mut Context
    ) -> Result<InletList<'a>, ockam_core::Error> {
        Ok(InletList::new(
        node_manager.registry
                .inlets
                .iter()
                .map(|(alias, info)| {
                    InletStatus::new(
                        &info.bind_addr,
                        info.worker_addr.to_string(),
                        alias,
                        None,
                        info.outlet_route.to_string(),
                    )
                })
                .collect(),
        ))
    }

    pub(crate) async fn build_outlets_list<'a>(
        &self,
        node_manager: &'a RwLockReadGuard<'_, NodeManager>,
        ctx: &mut Context
    ) -> Result<OutletList<'a>, ockam_core::Error> {
        Ok(OutletList::new(
        node_manager.registry
                .outlets
                .iter()
                .map(|(alias, info)| {
                    OutletStatus::new(&info.tcp_addr, info.worker_addr.to_string(), alias, None)
                })
                .collect()
        ))
    }
}