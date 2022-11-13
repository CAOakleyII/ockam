use ockam::Context;
use ockam_core::api::{Request, ResponseBuilder, Response};
use ockam_node::tokio::sync::RwLockReadGuard;

use crate::nodes::{NodeManagerWorker, NodeManager, models::secure_channel::SecureChannelList};

impl NodeManagerWorker {
    // TODO: Should have a model, instead of Vec<String>,
    // Should be able to use SecureChannelList?
    pub(crate) async fn get_secure_channel_listeners(
        &self,
        req: &Request<'_>,
        node_manager: &'_ RwLockReadGuard<'_, NodeManager>,
        ctx: &mut Context
    ) -> Result<ResponseBuilder<Vec<String>>, ockam_core::Error> {
        Ok(Response::ok(req.id()).body(
            self.build_secure_channel_listeners(node_manager, ctx).await?
        ))
    }

    pub(crate) async fn get_secure_channels<'a>(
        &self,
        req: &Request<'a>,
        node_manager: &'a RwLockReadGuard<'_, NodeManager>,
        ctx: &mut Context
    ) -> Result<ResponseBuilder<SecureChannelList<'a>>, ockam_core::Error> {
        Ok(Response::ok(req.id()).body(
            self.build_secure_channels_list(
                node_manager,
                ctx
            ).await?
        ))
    }
}