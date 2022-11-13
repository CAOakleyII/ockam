use std::sync::Arc;

use ockam::Context;
use ockam_core::api::{Status};
use ockam_identity::Identity;
use ockam_node::tokio::sync::RwLockReadGuard;
use ockam_vault::{storage::FileStorage, Vault};

use crate::{nodes::{NodeManagerWorker, NodeManager, models::identity::{ShortIdentityResponse, LongIdentityResponse, IdentityResponse}, overseer::Overseer}, config::cli::{self}};

impl NodeManagerWorker {

    /// Retrieves the identity model, with both short and long when possible.
    pub(crate) async fn retrieve_identity_response<'a>(
        &self,
        node_manager: &'a RwLockReadGuard<'_, NodeManager>,
        ctx: &mut Context
    ) -> Result<IdentityResponse<'a>, ockam_core::Error> {
        // TODO: Can have options to only retrieve certain identity responses (LONG  | SHORT)
        let short = match self.retrieve_short_identity_response(node_manager, ctx).await {
            Ok(l) => Some(l),
            Err(_) => None,
        };

        let long  = match self.retrieve_long_identity_response(node_manager, ctx).await {
            Ok(l) => Some(l),
            Err(_) => None,
        };

        Ok(IdentityResponse::new(
            short,
            long
        ))
    }

    /// Retrieves the short identity
    pub(crate) async fn retrieve_short_identity_response<'a>(
        &self,
        node_manager: &'a RwLockReadGuard<'_, NodeManager>,
        _ctx: &mut Context
    ) -> Result<ShortIdentityResponse<'a>, ockam_core::Error> {
        let identity = node_manager.identity()?;
        let identifier = identity.identifier();

        Ok(ShortIdentityResponse::new(identifier.to_string()))
    }

    /// Retrieves the long identity
    pub(crate) async fn retrieve_long_identity_response<'a>(
        &self,
        node_manager: &'a RwLockReadGuard<'_, NodeManager>,
        _ctx: &mut Context
    ) -> Result<LongIdentityResponse<'a>, ockam_core::Error> {
        let identity = node_manager.identity()?;
        let identity = identity.export().await?;
    
        Ok(LongIdentityResponse::new(identity))
    }
}

impl Overseer {

    /// Creates the default identity and vault 
    /// if they are not already existing.
    pub(crate) async fn create_default_identity_if_needed(
        &mut self,
        ctx: &Context
    ) ->  Result<(), Status> {
        // Get default root vault (create if needed)
        let default_vault_path = match self.get_default_vault_path().await {
            Some(p) => p,
            None => {
                let default_vault_path = cli::OckamConfig::directories()
                .config_dir()
                .join("default_vault.json");
    
                self.set_default_vault_path(Some(default_vault_path.clone())).await;
        
                default_vault_path
            }
        };
    
        let storage = match FileStorage::create(default_vault_path.clone()).await {
            Ok(s) => s,
            Err(_e) => {
                return Err(Status::InternalServerError)
            }
        };
        let vault = Vault::new(Some(Arc::new(storage)));
    
        // Get default root identity (create if needed)
        if self.get_default_identity().await.is_none() {
            let identity = match Identity::create(ctx, &vault).await {
                Ok(i) => i,
                Err(_e) => {
                    return Err(Status::InternalServerError)
                }
            };
            let exported_data = match identity.export().await {
                Ok(i) => i,
                Err(_e) => {
                    return Err(Status::InternalServerError)
                }
            };
            self.set_default_identity(Some(exported_data)).await;
        };

        if let Err(_e) = self.persist_config_updates().await {
            return Err(Status::InternalServerError)
        }
    
        Ok(())
    }
    
}