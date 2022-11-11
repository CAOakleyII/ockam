use std::{sync::Arc, path::PathBuf};

use minicbor::Decoder;
use ockam::{Worker, Context, Result, Routed, Address, TcpTransport, compat::asynchronous::RwLock, Route, TCP};
use ockam_abac::ParseError;
use ockam_core::api::{Request, Method, Response, Error};

use crate::config::{cli::{OckamConfig, self}, Config, atomic::AtomicUpdaterAsync};

use super::NODEMANAGER_ADDR;

/// A Ockam Worker that handles the creation, management and API messages to 
/// individual nodes.
pub struct Overseer {
    config_path: PathBuf,
    tcp_transport: TcpTransport,
    config: Arc<RwLock<OckamConfig>>,
}

impl Overseer {
    pub fn new(tcp_transport: TcpTransport) -> Self {
        // Load and extract config data
        // to load it within an async RwLock
        // Current implementation of Config is only sync.
        let directories = cli::OckamConfig::directories();
        let config_dir = directories.config_dir();
        let config = Config::<cli::OckamConfig>::load(config_dir, "config").expect("Unable to load config.");
        let config_path = config.config_path();
        let mut c = config.read().clone();
        c.directories = Some(OckamConfig::directories());

        Self {
            config_path,
            config: Arc::new(RwLock::new(c)),
            tcp_transport
        }
    }
    pub fn config(&mut self) -> &mut Arc<RwLock<OckamConfig>> {
        &mut self.config
    }

    /// Atomically update the configuration
    pub async fn persist_config_updates(&self) -> anyhow::Result<()> {
        AtomicUpdaterAsync::new(
            self.config_path.clone(),
            self.config.clone()
        )
        .run().await
    }

    /// Retrieve a clone of the default vault path
    pub async fn get_default_vault_path(&self) -> Option<PathBuf> {
        self.config.read().await.default_vault_path.clone()
    }

    /// Retrieve a clone of the defaeult identity
    pub async fn get_default_identity(&self) -> Option<Vec<u8>> {
        self.config.read().await.default_identity.clone()
    }

    /// Set the defaeult vault path
    pub async fn set_default_vault_path(&self, default_vault_path: Option<PathBuf>) {
        self.config.write().await.default_vault_path = default_vault_path
    }

    /// Set the default identity
    pub async fn set_default_identity(&self, default_identity: Option<Vec<u8>>) {
        self.config.write().await.default_identity = default_identity;
    }
}

#[ockam::worker]
impl Worker for Overseer {
    type Message = Vec<u8>;
    type Context = Context;

    /// Handle a message sent to this worker
    /// 
    /// Messages with an X-Node-Name Header Parameter are forwarded to the node with that name.
    ///
    /// Currently that node would need to be managed by this worker.
    async fn handle_message(&mut self, ctx: &mut Context, msg: Routed<Vec<u8>>) -> Result<()> {
        let mut dec = Decoder::new(msg.as_body());
        let req: Request = match dec.decode() {
            Ok(r) => r,
            Err(e) => {
                error!("Failed to decode request: {:?}", e);
                let err = ParseError::message("Failed to decode request.");
                return Err(ockam_core::Error::from(err))
            }
        };

        // Check for an X-Node-Name Header
        if let Some(params) = req.parameters() {
            if let Some(node_name) = params.x_node_name() {
                                
                let addr = {
                    let cfg = self.config().read().await;
                    let node_cfg = match cfg.nodes.get(&node_name) {
                        Some(cfg) => cfg,
                        None => {
                            info!("Node {} was not found", node_name);
                            let r =  Response::not_found(req.id()).to_vec()?;
                            return ctx.send(msg.return_route(), r).await
                        }
                    };

                    Address::from((TCP, node_cfg.addr().to_string()))
                };

                if let Err(e) = self.tcp_transport.connect(addr.address()).await {
                    error!("Could not TCP connect to {:?}. Error: {}", addr.address(), e);
                }

                info!("Message being rerouted to node {} at {:?}", node_name, addr);

                let mut route_to_node_mgr: Route = NODEMANAGER_ADDR.into();
                route_to_node_mgr.modify().prepend_route(addr.into());

                // send and receive using default timeout
                let resp_from_node_mgr: Vec<u8> = match ctx
                    .send_and_receive(route_to_node_mgr.clone(), msg.to_vec())
                    .await {
                        Ok(b) => b,
                        Err(e) => {
                            error!("Error received from node {}. Error: {}", node_name, e);
                            Response::internal_error(req.id()).to_vec()?
                        }
                    };
                
                return ctx.send(msg.return_route(), resp_from_node_mgr).await
            }
        }

        let response = match self.handle_request(ctx, &req, &mut dec).await {
            Ok(r) => r,
            Err(err) => {
                error! {
                    re     = %req.id(),
                    method = ?req.method(),
                    path   = %req.path(),
                    code   = %err.code(),
                    "failed to handle request"
                }
                let err =
                    Error::new(req.path()).with_message(format!("failed to handle request: {err}"));
                Response::internal_error(req.id()).body(err).to_vec()?
            }
        };

        ctx.send(msg.return_route(), response).await
    }
}

impl Overseer {
    /// Handle an API request that is meant for the overseer
    async fn handle_request(
        &mut self,
        ctx: &mut Context,
        req: &Request<'_>,
        dec: &mut Decoder<'_>,
    ) -> Result<Vec<u8>> {

        use Method::*;

        let path = req.path();
        let path_segments = req.path_segments::<5>();
        let method = match req.method() {
            Some(m) => m,
            None => {
                error!("Failed to parse request method.");
                return Ok(Response::bad_request(req.id())
                    .body(format!("Invalid request method."))
                    .to_vec()?)
            }
        };

        let response = match (method, path_segments.as_slice()) {
            (Get, ["nodes"]) => {
                match self.get_nodes(req).await {
                    Ok(r) => r.to_vec()?,
                    Err(r) => r.to_vec()?,
                }
            }

            (Post, ["node"]) => {
                match self.post_node(req, ctx, dec).await {
                    Ok(r) => r.to_vec()?,
                    Err(r) => r.to_vec()?,
                }
            }

            // Can now implement these calls, and keep it abstracted from CLI
            // GET * Transports
            // GET * Secure Channels

            _ => {
                // Not Found Response for a path that doesn't match a route
                warn!(%method, %path, "Called non existent endpoint");
                Response::not_found(req.id())
                    .body(format!("Path not found: {:?} {}", method, path))
                    .to_vec()?
            }
        };

        Ok(response)
    }
}
