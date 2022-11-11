use std::borrow::Cow;

use ockam::Context;
use ockam_node::tokio::sync::RwLockReadGuard;

use crate::nodes::{NodeManagerWorker, NodeManager, models::secure_channel::{SecureChannelList, ShowSecureChannelResponse}};

impl NodeManagerWorker { 
    pub(crate) async fn build_secure_channel_listeners<'a>(
        &self,
        node_manager: &'a RwLockReadGuard<'_, NodeManager>,
        _ctx: &mut Context
    ) -> Result<Vec<String>, ockam_core::Error> {
        let registry = &node_manager.registry;

        Ok(registry
            .secure_channel_listeners
            .iter()
            .map(|(addr, _)| addr.to_string())
            .collect()
        )
    }

    pub(crate) async fn build_secure_channels_list<'a>(
        &self,
        node_manager: &'a RwLockReadGuard<'_, NodeManager>,
        _ctx: &mut Context
    ) -> Result<SecureChannelList<'a>, ockam_core::Error> {
        Ok(SecureChannelList::new(
            node_manager
                .registry
                .secure_channels
                .list()
                .iter()
                .map(|sci| ShowSecureChannelResponse::new(
                    Some(sci),
                        Cow::from(
                            format!("/node/{}", node_manager.node_name)
                        )
                    )
                )
                .collect()
        ))
    }
}