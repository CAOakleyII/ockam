use ockam::Context;
use ockam_core::api::{Request, ResponseBuilder, Response};
use ockam_node::tokio::sync::RwLockReadGuard;

use crate::nodes::{NodeManagerWorker, NodeManager, models::portal::{InletList, OutletList}};

impl NodeManagerWorker {
    pub(crate) async fn get_inlets<'a>(
        &self,
        req: &Request<'a>,
        node_manager: &'a RwLockReadGuard<'_, NodeManager>,
        ctx: &mut Context
    ) -> Result<ResponseBuilder<InletList<'a>>, ockam_core::Error> {
        Ok(Response::ok(req.id()).body(
            self.build_inlets_list(node_manager, ctx).await?
        ))
    }

    pub(crate) async fn get_outlets<'a>(
        &self,
        req: &Request<'a>,
        node_manager: &'a RwLockReadGuard<'_, NodeManager>,
        ctx: &mut Context
    ) -> Result<ResponseBuilder<OutletList<'a>>, ockam_core::Error> {
        Ok(Response::ok(req.id()).body(
            self.build_outlets_list(node_manager, ctx).await?
        ))
    }
}
