use minicbor::Encode;
use ockam::compat::borrow::Cow;
use ockam::TcpTransport;
use ockam_api::nodes::OVERSEER_ADDR;
use ockam_core::api::{Method, Request, RequestBuilder, RequestParameters};

use crate::{init::OVERSEER_NODE_NAME, CommandGlobalOpts};

use super::{node_rpc, Rpc, RpcBuilder};

/// Helps build an Ockam API Request
///
/// #### Example
///
/// ```rs
/// let api_builder = ApiBuilder::new(Method::Get)
///
/// api_builder
///     .to_path("node")
///     .to_path("transports")
///     .for_node("relay")
///     .exec_with_body(payload, options, handle_response)`
/// ```
#[derive(Debug)]
pub struct ApiBuilder {
    method: Method,
    path: Vec<String>,
    node_name: Option<String>,
    request_protocol: RequestProtcol,
}

#[derive(Debug)]
pub enum RequestProtcol {
    OckamRouting,
}

impl ApiBuilder {
    pub fn new(method: Method) -> Self {
        Self {
            method,
            path: Vec::new(),
            node_name: None,
            request_protocol: RequestProtcol::OckamRouting,
        }
    }

    /// Builds upon the path with another segment
    pub fn to_path<'a>(&'a mut self, path: String) -> &'a mut Self {
        self.path.push(path);
        self
    }

    /// Indicates a node for the request to be sent to
    pub fn for_node<'a>(&'a mut self, node_name: String) -> &'a mut Self {
        self.node_name = Some(node_name);
        self
    }

    /// Builds and executes the request
    ///
    /// Requires a body to be provided, to execute without a body use `exec`
    ///
    /// #### Arguments
    ///
    /// * `f` - A function that will be called with the `rpc` containing the response.
    pub fn exec_with_body<'a, T, F>(&'a mut self, b: T, opts: CommandGlobalOpts, f: F)
    where
        T: Encode<()> + Send + Sync + 'static,
        F: FnOnce(Rpc) + Send + Sync + 'static,
    {
        // ! could be extended to exec requests over other protocols
        // ! Though node--[ockam-router]-->node works pretty well.
        match self.request_protocol {
            RequestProtcol::OckamRouting => node_rpc(
                Self::node_rpc_exec,
                (
                    opts,
                    OVERSEER_NODE_NAME.to_string(),
                    self.build().body(b),
                    f,
                ),
            ),
        }
    }

    /// Builds and executes the request
    ///
    /// In order to provide a payload, use `exec_with_body`
    pub fn exec<'a, F>(&'a mut self, opts: CommandGlobalOpts, f: F)
    where
        F: FnOnce(Rpc) + Send + Sync + 'static,
    {
        // ! could be extended to exec requests over other protocols
        // ! Though node--[ockam-router]-->node works well.
        match self.request_protocol {
            RequestProtcol::OckamRouting => node_rpc(
                Self::node_rpc_exec,
                (opts, OVERSEER_NODE_NAME.to_string(), self.build(), f),
            ),
        }
    }

    fn build<'a>(&self) -> RequestBuilder<'a> {
        let path = self.path.join("/");
        let mut req_builder = match self.method {
            Method::Get => Request::get(path),
            Method::Post => Request::post(path),
            Method::Put => Request::put(path),
            Method::Delete => Request::delete(path),
            Method::Patch => Request::patch(path),
        };

        if let Some(node_name) = &self.node_name {
            req_builder = req_builder.parameters(Some(RequestParameters::new(Some(Cow::from(
                node_name.to_string(),
            )))));
        }

        req_builder
    }

    async fn node_rpc_exec<T, F>(
        ctx: ockam::Context,
        (opts, node_name, req_builder, f): (CommandGlobalOpts, String, RequestBuilder<'_, T>, F),
    ) -> crate::Result<()>
    where
        T: Encode<()> + Send + Sync + 'static,
        F: FnOnce(Rpc) + Send + Sync + 'static,
    {
        let tcp = TcpTransport::create(&ctx).await?;
        let mut rpc = RpcBuilder::new_to(&ctx, &opts, &node_name, OVERSEER_ADDR.into())
            .tcp(&tcp)?
            .build();

        rpc.request(req_builder).await?;
        f(rpc);

        Ok(())
    }
}
