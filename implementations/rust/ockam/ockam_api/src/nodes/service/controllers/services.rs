use ockam::Context;
use ockam_core::api::{Request, Response, ResponseBuilder};
use ockam_node::tokio::sync::RwLockReadGuard;

use crate::nodes::{NodeManagerWorker, NodeManager, models::services::ServiceList};

impl NodeManagerWorker {
    pub(crate) async fn get_services<'a>(
        &self,
        req: &Request<'a>,
        node_manager: &'a RwLockReadGuard<'_, NodeManager>,
        ctx: &mut Context
    ) -> Result<ResponseBuilder<ServiceList<'a>>, ockam_core::Error> {
        Ok(Response::ok(req.id()).body(
            self.retrieve_service_list(node_manager, ctx).await?
        ))
    }
}