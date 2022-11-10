use ockam::Context;
use ockam_core::api::{Request, ResponseBuilder, Response};
use ockam_node::tokio::sync::RwLockReadGuard;

use crate::nodes::{NodeManagerWorker, NodeManager, models::identity::IdentityResponse};

impl NodeManagerWorker {
    pub(crate) async fn get_identity<'a>(
        &self,
        req: &Request<'a>,
        node_manager: &'a RwLockReadGuard<'_, NodeManager>,
        ctx: &mut Context
    ) -> Result<ResponseBuilder<IdentityResponse<'a>>, ockam_core::Error> {

        let response =
            Response::ok(req.id()).body(
                self.build_identity_response(&node_manager, ctx).await?
            );
        Ok(response)
    }
}
