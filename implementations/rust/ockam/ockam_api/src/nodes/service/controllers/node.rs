use minicbor::Decoder;
use ockam::Context;
use ockam_core::api::{Request, ResponseBuilder, Response};
use ockam_node::tokio::sync::RwLockReadGuard;
use crate::nodes::{models::{base::{NodeStatus, GetNodeStatusRequest}}, NodeManagerWorker, NodeManager};


impl NodeManagerWorker {
    // TODO: Thoughts on handling routing
    // #[get('/node')] or
    // router_builder.get('/node')
    pub(crate) async fn get_node<'a, 'b>(
        &'b self,
        req: &'a Request<'_>,
        node_manager: &'a RwLockReadGuard<'b, NodeManager>,
        ctx: &mut Context,
        dec: &mut Decoder<'_>,
    ) -> Result<ResponseBuilder<NodeStatus<'a>>, ockam_core::Error> {
        let GetNodeStatusRequest {
            detailed
        } = dec.decode()?;

        let d = match detailed {
            Some(v) => v,
            None => false
        };

        Ok(Response::ok(req.id()).body(
            self.build_node_status(node_manager, ctx, d).await?
        ))
    }
}