use std::path::PathBuf;

use clap::{command, Args};
use colorful::Colorful;
use ockam_api::nodes::models::base::{CreateNodeRequest, NodeStatus};
use ockam_multiaddr::proto::Node;
use ockam_multiaddr::MultiAddr;
use rand::random;

use crate::help;
use crate::util::Rpc;
use crate::{util::api_builder::ApiBuilder, CommandGlobalOpts};

use crate::get::node::NODE_HELP_DETAIL;

/// Command to create a node resource
#[derive(Clone, Debug, Args)]
#[command(
    arg_required_else_help = false,
    after_long_help = help::template(NODE_HELP_DETAIL)
)]
pub struct NodeCommand {
    /// Name of the node (Optional).
    #[arg(hide_default_value = true, default_value_t = hex::encode(&random::<[u8;4]>()))]
    pub node_name: String,

    /// TCP listener address
    #[arg(
        display_order = 900,
        long,
        short,
        id = "SOCKET_ADDRESS",
        default_value = "127.0.0.1:0"
    )]

    /// TCP listener address
    #[arg(
        display_order = 900,
        long,
        short,
        id = "SOCKET_ADDRESS",
        default_value = "127.0.0.1:0"
    )]
    pub tcp_listener_address: String,

    /// Skip creation of default Vault and Identity
    #[arg(long, short, hide = true)]
    pub skip_defaults: bool,

    /// Skip credential checks
    #[arg(long, hide = true)]
    pub enable_credential_checks: bool,

    /// Don't share default identity with this node
    #[arg(long, hide = true)]
    pub no_shared_identity: bool,

    /// JSON config to setup a foreground node
    ///
    /// This argument is currently ignored on background nodes.  Node
    /// configuration is run asynchronously and may take several
    /// seconds to complete.
    #[arg(long, hide = true)]
    pub launch_config: Option<PathBuf>,

    #[arg(long, hide = true)]
    pub no_watchdog: bool,

    #[arg(long, hide = true)]
    pub project: Option<PathBuf>,

    #[arg(long, hide = true)]
    pub config: Option<PathBuf>,
}

impl Default for NodeCommand {
    fn default() -> Self {
        Self {
            node_name: hex::encode(&random::<[u8; 4]>()),
            tcp_listener_address: "127.0.0.1:0".to_string(),
            skip_defaults: false,
            enable_credential_checks: false,
            no_shared_identity: false,
            launch_config: None,
            no_watchdog: false,
            project: None,
            config: None,
        }
    }
}

/// Command to create a node resource
impl NodeCommand {
    pub fn run(self, api_builder: &mut ApiBuilder, options: CommandGlobalOpts) {
        let payload = CreateNodeRequest::new(
            self.node_name,
            self.tcp_listener_address,
            self.skip_defaults,
            self.enable_credential_checks,
            self.no_shared_identity,
            self.launch_config,
            self.no_watchdog,
            self.project,
            self.config,
            options.global_args.verbose,
        );

        api_builder
            .to_path("node".to_string())
            .exec_with_body(payload, options, print_response);
    }
}

fn print_response(rpc: Rpc) {
    let resp = rpc.parse_response::<NodeStatus>();
    match resp {
        Ok(node_status) => print_node_status(&node_status),
        Err(e) => println!("Error creating node: {:?}", e),
    }
}

fn print_node_status(node_status: &NodeStatus) {
    println!();
    println!("Node:");
    println!("  Name: {}", node_status.node_name);
    println!("  Status: {}", node_status.status.yellow());

    println!("  Route To Node:");
    let mut m = MultiAddr::default();
    if m.push_back(Node::new(node_status.node_name.clone()))
        .is_ok()
    {
        println!("    Short: {}", m);
    }
}
