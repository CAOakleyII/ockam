use minicbor::Decoder;
use ockam::Context;
use ockam_core::{api::{Request, Response, ResponseBuilder}};
use ockam_node::tokio::sync::RwLockReadGuard;

use crate::nodes::{NodeManagerWorker, NodeManager, models::transport::{TransportList, GetTransportList, TransportMode, TransportType}};
impl NodeManagerWorker {
    pub(crate) async fn get_transports<'a>(
        &self,
        req: &Request<'a>,
        node_manager: &'a RwLockReadGuard<'_, NodeManager>,
        ctx: &mut Context,
        dec: &mut Decoder<'_>,
    ) -> Result<ResponseBuilder<TransportList<'a>>, ockam_core::Error> {
        let GetTransportList {
            tts,
            tms
        } = dec.decode()?;
    
        let tts_vec: Vec<TransportType> = match tts {
            Some(tts) => tts,
            None => vec![TransportType::Tcp, TransportType::Ble, TransportType::WebSocket]
        };
    
        let tms_vec: Vec<TransportMode> = match tms {
            Some(tms) => tms,
            None => vec![TransportMode::Listen, TransportMode::Connect]
        };
    
        Ok(Response::ok(req.id()).body(
            self.retrieve_transport_list(node_manager, ctx, tts_vec, tms_vec).await?
        ))
    }
}