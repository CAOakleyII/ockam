use ockam::Context;
use ockam_node::tokio::sync::RwLockReadGuard;

use crate::nodes::{NodeManagerWorker, NodeManager, models::identity::{ShortIdentityResponse, LongIdentityResponse, IdentityResponse}};

impl NodeManagerWorker {
    pub(crate) async fn build_identity_response<'a>(
        &self,
        node_manager: &'a RwLockReadGuard<'_, NodeManager>,
        ctx: &mut Context
    ) -> Result<IdentityResponse<'a>, ockam_core::Error> {
        // TODO: Can have options to only retrieve certain identity responses (LONG  | SHORT)
        let short = match self.build_short_identity_response(node_manager, ctx).await {
            Ok(l) => Some(l),
            Err(_) => None,
        };

        let long  = match self.build_long_identity_response(node_manager, ctx).await {
            Ok(l) => Some(l),
            Err(_) => None,
        };

        Ok(IdentityResponse::new(
            short,
            long
        ))
    }

    pub(crate) async fn build_short_identity_response<'a>(
        &self,
        node_manager: &'a RwLockReadGuard<'_, NodeManager>,
        ctx: &mut Context
    ) -> Result<ShortIdentityResponse<'a>, ockam_core::Error> {
        let identity = node_manager.identity()?;
        let identifier = identity.identifier();

        Ok(ShortIdentityResponse::new(identifier.to_string()))
    }

    pub(crate) async fn build_long_identity_response<'a>(
        &self,
        node_manager: &'a RwLockReadGuard<'_, NodeManager>,
        ctx: &mut Context
    ) -> Result<LongIdentityResponse<'a>, ockam_core::Error> {
        let identity = node_manager.identity()?;
        let identity = identity.export().await?;
    
        Ok(LongIdentityResponse::new(identity))
    }
}