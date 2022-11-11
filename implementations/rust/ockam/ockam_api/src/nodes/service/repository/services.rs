use ockam::Context;
use ockam_node::tokio::sync::RwLockReadGuard;

use crate::nodes::{NodeManagerWorker, NodeManager, models::services::{ServiceList, ServiceStatus}};

impl NodeManagerWorker {
    pub(crate) async fn build_service_list<'a>(
        &self,
        node_manager: &'a RwLockReadGuard<'_, NodeManager>,
        _ctx: &mut Context
    ) -> Result<ServiceList<'a>, ockam_core::Error> {
        let mut list = Vec::new();
        let registry = &node_manager.registry;

        registry
            .vault_services
            .keys()
            .for_each(|addr| list.push(ServiceStatus::new(addr.address(), "vault")));
        registry
            .identity_services
            .keys()
            .for_each(|addr| list.push(ServiceStatus::new(addr.address(), "identity")));
        registry
            .authenticated_services
            .keys()
            .for_each(|addr| list.push(ServiceStatus::new(addr.address(), "authenticated")));
        registry
            .uppercase_services
            .keys()
            .for_each(|addr| list.push(ServiceStatus::new(addr.address(), "uppercase")));
        registry
            .echoer_services
            .keys()
            .for_each(|addr| list.push(ServiceStatus::new(addr.address(), "echoer")));
        registry
            .verifier_services
            .keys()
            .for_each(|addr| list.push(ServiceStatus::new(addr.address(), "verifier")));
        registry
            .credentials_services
            .keys()
            .for_each(|addr| list.push(ServiceStatus::new(addr.address(), "credentials")));

        #[cfg(feature = "direct-authenticator")]
        registry
            .authenticator_service
            .keys()
            .for_each(|addr| list.push(ServiceStatus::new(addr.address(), "authenticator")));

        Ok(ServiceList::new(list))
    }
}