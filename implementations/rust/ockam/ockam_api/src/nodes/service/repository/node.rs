
use ockam_identity::Identity;
use ockam_node::tokio::sync::RwLockReadGuard;
use ockam_vault::{storage::FileStorage, Vault};
use std::{net::SocketAddr, sync::Arc, env::current_exe, fs::OpenOptions, process::Command};

use ockam::Context;
use slug::slugify;
use ockam_core::api::Status;
use ockam_node::tokio::fs::create_dir_all;
use crate::{nodes::{models::{base::{NodeStatus, NodeDetails, CreateNodeRequest}, transport::{TransportType, TransportMode}}, NodeManagerWorker, NodeManager, config::NodeConfig, overseer::Overseer}, config::cli::{NodeConfigOld, OckamConfig, self}};

impl NodeManagerWorker {
    pub(crate) async fn build_node_status<'a, 'b>(
        &'b self,
        node_manager: &'a RwLockReadGuard<'b, NodeManager>,
        ctx: &mut Context,
        details: bool,
    ) -> Result<NodeStatus<'a>, ockam_core::Error> {
        // Retrieve mode details if requested
        let node_details = if details {
            Some(NodeDetails::new(
                "Running",
                self.build_short_identity_response(node_manager, ctx).await?,
                self.build_service_list(node_manager, ctx).await?,
                self.build_transport_list(node_manager, ctx, vec![TransportType::Tcp], vec![TransportMode::Listen]).await?,
                self.build_secure_channel_listeners(node_manager, ctx).await?,
                self.build_inlets_list(node_manager, ctx).await?,
                self.build_outlets_list(node_manager, ctx).await?,
            ))
        } else {
            None
        };

       Ok(NodeStatus::new(
            &node_manager.node_name,
            "Running",
            ctx.list_workers().await?.len() as u32,
            std::process::id() as i32,
            node_manager.transports.len() as u32,
            node_details
        ))
    }
}

impl Overseer {

    /// Creates a node configuration within the OckamConfig 
    /// and configuration storage.
    pub (crate) async fn create_node(
        &mut self,
        node_name: &String,
        bind: SocketAddr,
        verbose: u8
    ) -> Result<(), Status> {
        {
            let mut cfg = self.config().write().await;

            // Check if any nodes have the same name.
            if cfg.nodes.contains_key(node_name) {
                error!("Error node name, {}, is already in use.", node_name);
                return Err(Status::Conflict)
            }
        
            // Check if the port is used by some other services or process
            let port = bind.port().clone();
            if !bind_to_port_check(&bind) || 
                cfg.nodes.iter().any(|(_, n)| n.port() == port) 
            {
                error!("Port, {}, is already in use.", bind.port());
                return Err(Status::Conflict)
            }
        
            let state_dir = match cfg
                .directories
                .as_ref() {
                    Some(v) => v,
                    None => {
                        error!("Error loading configuration. cfg state: {:?}", cfg);
                        return Err(Status::BadRequest)
                    }
                }
                .data_local_dir()
                .join(slugify(&format!("node-{}", node_name)));
        
        
            if let Err(e) = create_dir_all(&state_dir).await {
                error!("Error creating state directory: {:?}. Error: {}", &state_dir, e);
                return Err(Status::InternalServerError)
            }
        
            // Initialize it
            if let Err(e) = NodeConfig::init_for_new_node(&state_dir) {
                error!("Error initializing new node for state directory: {:?}. Error: {}", &state_dir, e);
                return Err(Status::InternalServerError)
            }
        
            // Add this node to the config lookup table
            cfg.lookup.set_node(node_name, bind.into());
        
            // Set First Created Node as Default Node
            if cfg.default.is_none() {
                cfg.default = Some(node_name.to_string());
            }
        
            // Add this node to the main node table
            cfg.nodes.insert(
                node_name.to_string(),
                NodeConfigOld::new(
                    node_name.to_string(),
                    bind.into(),
                    bind.port(),
                    verbose,
                    None,
                    Some(state_dir),
                ),
            );
        }

        if let Err(e) = self.persist_config_updates().await {
            return Err(Status::InternalServerError)
        }
        
        Ok(())
    }

    /// Spawn the node by rerunning the CLI command
    /// but with a foreground flag
    /// 
    /// TODO: Thoughts on improving this would have a seperate binary, that starts
    /// a node with the Ockam Services.
    pub(crate) async fn spawn_node(
        &mut self,
        req_body: CreateNodeRequest,
        tcp_listener_address: String,
    ) -> Result<(), Status> {
        {
            let mut cfg = self.config().write().await;
            let ockam_exe = current_exe().unwrap_or_else(|_| "ockam".into());
            let node_name = &req_body.node_name;

            let node_cfg = match cfg.nodes.get(node_name) {
                Some(v) => v,
                None => {
                    error!("Node with the name of {} was not found within the configuration.", node_name);
                    return Err(Status::NotFound)
                }
            };
            let base = match node_cfg.state_dir() {
                Some(v) => v,
                None => {
                    error!("Unable to retrieve state directory.");
                    return Err(Status::InternalServerError)
                },
            };

            let (mlog, elog) = (
                base.join(format!("{}.log", node_name)),
                base.join(format!("{}.log.stderr", node_name)),
            );

            let main_log_file = match OpenOptions::new()
                .create(true)
                .append(true)
                .open(mlog) {
                    Ok(f) => f,
                    Err(e) => {
                        error!("Failed to open main log path. {}", e);
                        return Err(Status::InternalServerError)
                    }
                };

            let stderr_log_file = match OpenOptions::new()
                .create(true)
                .append(true)
                .open(elog) {
                    Ok(f) => f,
                    Err(e) => {
                        error!("Failed to open stderr log path. {}", e);
                        return Err(Status::InternalServerError)
                    }
                };
                

            let mut args = vec![
                match req_body.verbose {
                    0 => "-vv".to_string(),
                    v => format!("-{}", "v".repeat(v as usize)),
                },
                "--no-color".to_string(),
                "node".to_string(),
                "create".to_string(),
                "--tcp-listener-address".to_string(),
                tcp_listener_address,
                "--foreground".to_string(),
                "--child-process".to_string(),
            ];

            if let Some(path) = req_body.project {
                args.push("--project".to_string());
                let p = match path
                    .to_str() {
                        Some(p) => p,
                        None => {
                            error!("Unsupported project path {:?}", path);
                            return Err(Status::BadRequest)
                        }
                    };
                args.push(p.to_string())
            }

            if req_body.skip_defaults {
                args.push("--skip-defaults".to_string());
            }

            if req_body.no_shared_identity {
                args.push("--no-shared-identity".to_string());
            }

            if req_body.enable_credential_checks {
                args.push("--enable-credential-checks".to_string());
            }

            args.push(node_name.to_owned());

            let child = match Command::new(ockam_exe)
                .args(args)
                .stdout(main_log_file)
                .stderr(stderr_log_file)
                .spawn() {
                    Ok(c) => c,
                    Err(e) => {
                        error!("Error spawning node process: {}", e);
                        return Err(Status::InternalServerError);
                    }
                };

            cfg.nodes.get_mut(node_name).unwrap().pid = Some(child.id() as i32);
        }

        if let Err(e) = self.persist_config_updates().await {
            return Err(Status::InternalServerError)
        }
    
        Ok(())
    }
}



fn bind_to_port_check(address: &SocketAddr) -> bool {
    let port = address.port();
    let ip = address.ip();
    std::net::TcpListener::bind((ip, port)).is_ok()
}