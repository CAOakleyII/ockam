use ockam::Context;
use ockam_node::tokio::sync::RwLockReadGuard;

use crate::nodes::{NodeManagerWorker, NodeManager, models::transport::{TransportList, TransportMode, TransportType, TransportStatus}};

impl NodeManagerWorker {
    pub(crate) async fn build_transport_list<'a>(
        &self,
        node_manager: &'a RwLockReadGuard<'_, NodeManager>,
        _ctx: &mut Context,
        tts: Vec<TransportType>,
        tms: Vec<TransportMode>
    ) -> Result<TransportList<'a>, ockam_core::Error> {
        Ok(TransportList::new(
            node_manager.transports
                .iter()
                .filter(|(_, (tt, tm, _))| tts.contains(tt) && tms.contains(tm) )
                .map(|(tid, (tt, tm, addr))| TransportStatus::new(*tt, *tm, addr, tid))
                .collect(),
        ))
    }
}