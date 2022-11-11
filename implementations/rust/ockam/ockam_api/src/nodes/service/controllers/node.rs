use std::{net::{SocketAddr, IpAddr, TcpListener, AddrParseError}, str::FromStr};

use minicbor::Decoder;
use ockam::Context;
use slug::slugify;
use ockam_core::api::{Request, ResponseBuilder, Response, Status};
use ockam_node::tokio::{sync::RwLockReadGuard, fs::create_dir_all};
use crate::{nodes::{models::{base::{NodeStatus, GetNodeStatusRequest, NodeList, CreateNodeRequest}}, NodeManagerWorker, NodeManager, overseer::Overseer, config::NodeConfig}, config::cli::{NodeConfigOld, OckamConfig}};


impl NodeManagerWorker {
    // TODO: Thoughts on handling routing
    // #[get('/node')] or
    // router_builder.get('/node')

    /// `GET /node`
    /// 
    /// Request Handler
    /// 
    /// Retrieves the `NodeStatus` of the current node.
    pub(crate) async fn get_node<'a, 'b>(
        &'b self,
        req: &'a Request<'_>,
        node_manager: &'a RwLockReadGuard<'b, NodeManager>,
        ctx: &mut Context,
        dec: &mut Decoder<'_>,
    ) -> Result<ResponseBuilder<NodeStatus<'a>>, ResponseBuilder> {
        let req_body: GetNodeStatusRequest = match dec.decode() {
            Ok(req_body) => req_body,
            Err(e) => {
                error!(?e, "Error decoding request body for request: {:?}", req);
                return Err(Response::bad_request(req.id()))
            }

        };

        let d = match req_body.detailed {
            Some(v) => v,
            None => false
        };

        let node_status: NodeStatus = match self.build_node_status(node_manager, ctx, d).await {
            Ok(node) => node,
            Err(e) => {
                error!(?e, "Error building node status");
                return Err(Response::internal_error(req.id()))
            }
        };

        Ok(Response::ok(req.id()).body(
            node_status
        ))
    }
}

impl Overseer {

    /// `GET /nodes`
    /// 
    /// Request handler
    /// 
    /// Retrieves a `NodeList` of current nodes being managed
    pub(crate) async fn get_nodes<'a>(
        &'a mut self,
        req: &'a Request<'_>,
    ) -> Result<ResponseBuilder<NodeList<'a>>, ResponseBuilder> {
        let cfg: RwLockReadGuard<'a, OckamConfig> = self.config().read().await;

        let values: Vec<NodeStatus> = cfg.nodes
            .iter()
            .map(|(name, node_config)| {
                // Oakley - TODO: this needs to be pulled from a call to the node to get this info
                // Node -- API --> Node call support is needed
                NodeStatus::new(
                    name.to_string(),
                    "Up",
                    0 as u32,
                    node_config.pid().unwrap() as i32,
                    0 as u32,
                    None
                )
            })
            .collect();
        
        let response = NodeList::new(values);

        Ok(Response::ok(req.id()).body(
          response  
        ))
    }

    /// `POST /node` 
    /// 
    /// Request Handler
    /// 
    /// Spawns a new node as a background process that will be managed by this worker
    pub(crate) async fn post_node<'a, 'b>(
        &'b mut self,
        req: &'a Request<'_>,
        ctx: &mut Context,
        dec: &mut Decoder<'_>,
    ) -> Result<ResponseBuilder<NodeStatus<'a>>, ResponseBuilder> {
        let req_body: CreateNodeRequest =  match dec.decode() {
            Ok(v) => v,
            Err(e) => {
                error!("Error decoding request body for request: {:?}. Error: {}", req, e);
                return Err(Response::bad_request(req.id()))
            }
        };

        let node_name = &req_body.node_name;
        let tcp_address = match Self::get_open_tcp_address(&req_body.tcp_listener_address) {
            Ok (a) => a,
            Err(e) => return Err(Response::builder(req.id(), e))
        };

        let bind = match SocketAddr::from_str(&tcp_address) {
            Ok(v) => v,
            Err(e) => { 
                error!("Error parsing tcp listener address: {}. Error: {}", &req_body.tcp_listener_address, e);
                return Err(Response::bad_request(req.id()))
            }
        };

        {
            if let Err(e) = self.create_node(node_name, bind, req_body.verbose).await {
                return Err(Response::builder(req.id(), e))
            };
        }

        {
            if let Err(e) = self.create_default_identity_if_needed(ctx).await {
                return Err(Response::builder(req.id(), e))
            }    
        }

        {
            // spawn node by rerunning the CLI command but with a different flag, to run as foreground
            // TODO: This should be improved, if we could have a seperate binary on installation for for just running a node at foreground
            if let Err(e) = self.spawn_node(req_body.clone(), tcp_address).await {
                return Err(Response::builder(req.id(), e));
            }
        }


        let node_status: NodeStatus<'a> = NodeStatus::new(
            node_name.to_string(),
            "Pending",
            0 as u32,
            0 as i32, 
            0 as u32,
            None
        );

        Ok(Response::ok(req.id())
            .body(
                node_status
            )
        )
    }

    /// Helper method to find an open tcp address:port where the overseer is running
    fn get_open_tcp_address(tcp_listener_address: &str) -> Result<String, Status> {
        let addr: SocketAddr = if tcp_listener_address == "127.0.0.1:0" {
            let port = match Self::find_available_port(){
                Ok(p) => p,
                Err(_) => return Err(Status::InternalServerError)
            };
            let ip_addr = match IpAddr::from_str("127.0.0.1") {
                Ok(ip) => ip,
                Err(_) => return Err(Status::InternalServerError)
            };
            SocketAddr::new(ip_addr, port)
        } else {
            match tcp_listener_address.parse() {
                Ok(a) => a,
                Err(_) => return Err(Status::BadRequest)
            }
        };

        Ok(addr.to_string())
    }

    /// Helper method to find an open tcp port where the overseer is running
    fn find_available_port() -> Result<u16, Status> {
        let listener = match TcpListener::bind("127.0.0.1:0") {
            Ok(tcp) => tcp,
            Err(_) => return Err(Status::InternalServerError)
        };

        let address = match listener.local_addr() {
            Ok(addr) => addr,
            Err(_) => return Err(Status::InternalServerError)
        };

        Ok(address.port())
    }
}