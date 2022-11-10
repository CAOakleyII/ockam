use ockam::Context;
use ockam_node::tokio::sync::RwLockReadGuard;
use crate::nodes::{models::{base::{NodeStatus, NodeDetails}, transport::{TransportMode, TransportType}}, NodeManagerWorker, NodeManager};

impl NodeManagerWorker {
    pub(crate) async fn build_node_status<'a, 'b>(
        &'b self,
        node_manager: &'a RwLockReadGuard<'b, NodeManager>,
        ctx: &mut Context,
        details: bool,
    ) -> Result<NodeStatus<'a>, ockam_core::Error> {
        // Retrieve mode details if requested
        let node_details = if details {
            Some(NodeDetails::new(
                "Running",
                self.build_short_identity_response(node_manager, ctx).await?,
                self.build_service_list(node_manager, ctx).await?,
                self.build_transport_list(node_manager, ctx, vec![TransportType::Tcp], vec![TransportMode::Listen]).await?,
                self.build_secure_channel_listeners(node_manager, ctx).await?,
                self.build_inlets_list(node_manager, ctx).await?,
                self.build_outlets_list(node_manager, ctx).await?,
            ))
        } else {
            None
        };

       Ok(NodeStatus::new(
            &node_manager.node_name,
            "Running",
            ctx.list_workers().await?.len() as u32,
            std::process::id() as i32,
            node_manager.transports.len() as u32,
            node_details
        ))
    }
}