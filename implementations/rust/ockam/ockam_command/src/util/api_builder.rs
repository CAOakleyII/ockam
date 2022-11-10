use minicbor::Encode;
use ockam::TcpTransport;
use ockam_core::api::{Request, RequestBuilder, Method};

use crate::CommandGlobalOpts;

use super::{node_rpc, RpcBuilder, Rpc};

#[derive(Debug)]
pub struct ApiBuilder {
    method: Method,
    path: Vec<String>,
    node_name: Option<String>,
}

impl ApiBuilder {
    pub fn new(method: Method) -> Self {
        Self {
            method,
            path: Vec::new(),
            node_name: None,
        }
    }

    pub fn to_path<'a>(&'a mut self, path: String) -> &'a mut Self {
        self.path.push(path);
        self
    }

    pub fn rpc_to_node<'a> (
        &'a mut self,
        node_name: String 
    ) -> &'a mut Self {
        self.node_name = Some(node_name);
        self
    }

    pub fn exec_with_body<'a, T, F>(
        &'a mut self,
        b: T,
        opts: CommandGlobalOpts,
        f: F
    )
    where
        T: Encode<()> + Send + Sync + 'static,
        F: FnOnce(Rpc) + Send + Sync + 'static
    {
        let path = self.path.join("/");
        let req_builder = match self.method {
            Method::Get => Request::get(path),
            Method::Post => Request::post(path),
            Method::Put => Request::put(path),
            Method::Delete => Request::delete(path),
            Method::Patch => Request::patch(path),
        };

        let req_body_builder = req_builder.body(b);

        match &self.node_name {
            Some(node_name) => node_rpc(
                Self::node_rpc_exec, 
                (opts, node_name.clone(), req_body_builder, f)
            ),
            None => println!("API calls currently must be ran against an RPC."),
        }
    }

    pub fn exec<'a, F>(
        &'a mut self,
        opts: CommandGlobalOpts,
        f: F
    )
    where
        F: FnOnce(Rpc) + Send + Sync + 'static
    {
        let path = self.path.join("/");
        let req_builder = match self.method {
            Method::Get => Request::get(path),
            Method::Post => Request::post(path),
            Method::Put => Request::put(path),
            Method::Delete => Request::delete(path),
            Method::Patch => Request::patch(path),
        };

        match &self.node_name {
            Some(node_name) => node_rpc(
                Self::node_rpc_exec, 
                (opts, node_name.clone(), req_builder, f)
            ),
            None => println!("API calls currently must be ran against an RPC."),
        }
    }

    async fn node_rpc_exec<T, F>(
        ctx: ockam::Context,
        (opts, node_name, req_builder, f): (CommandGlobalOpts, String, RequestBuilder<'_, T>, F)
    ) -> crate::Result<()>
    where
        T: Encode<()> + Send + Sync + 'static,
        F: FnOnce(Rpc) + Send + Sync + 'static
    {
        let tcp = TcpTransport::create(&ctx).await?;
        let mut rpc = RpcBuilder::new(&ctx, &opts, &node_name).tcp(&tcp)?.build();

        rpc.request(req_builder).await?;
        f(rpc);

        Ok(())
    }
}