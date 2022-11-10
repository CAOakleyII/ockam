use clap::{Args, Subcommand};

mod transports;
mod secure_channels;

use colorful::Colorful;
use ockam::Route;
use ockam_api::nodes::models::transport;
use ockam_api::{addr_to_multiaddr, route_to_multiaddr};
use ockam_api::nodes::models::base::{NodeStatus, NodeDetails, GetNodeStatusRequest};
use ockam_multiaddr::MultiAddr;
use ockam_multiaddr::proto::{Node, DnsAddr, Tcp};

use crate::secure_channel::SecureChannelCommand;
use crate::util::Rpc;
use crate::util::api_builder::ApiBuilder;
use crate::{help, CommandGlobalOpts};
use crate::node::NodeOpts;

use self::secure_channels::SecureChannelsCommand;
use self::transports::TransportsCommand;

const HELP_DETAIL: &str ="\
About:
    An Ockam node is any running application that can communicate with other applications
    using various Ockam protocols like Routing, Secure Channels, Forwarding etc.

    We can create Ockam nodes using this command line or using various Ockam programming
    libraries like our Rust and Elixir libraries.

    Workers
    ------

    Ockam nodes run very lightweight, concurrent, stateful actors called Ockam Workers.
    Workers have addresses and a node can deliver messages to workers on the same node or
    on a different node using the Ockam Routing Protocol and its Transports.


    Routing
    ------

    The Ockam Routing Protocol is a very simple application layer protocol that allows
    the sender of a message to describe the `onward_route` and `return_route` of message.

    The routing layer in a node can then be used to route these messages between workers within
    a node or across nodes using transports. Messages can be sent over multiple hops, within
    one node or across many nodes.


    Transports
    ------

    Transports are plugins to the Ockam Routing layer that allow Ockam Routing messages
    to travel across nodes over transport layer protocols like TCP, UDP, BLUETOOTH etc.


    Services
    ------

    One or more Ockam Workers can work as a team to offer a Service. Services have
    addresses represented by /service/{ADDRESS}. Services can be attached to identities and
    authorization policies to enforce attribute based access control rules.

    Nodes created using `ockam` command usually start a pre-defined set of default services.

    This includes:
        - A uppercase service at /service/uppercase
        - A secure channel listener at /service/api
        - A tcp listener listening at some TCP port
";

/// List all current nodes
#[derive(Clone, Debug, Args)]
#[command(
    arg_required_else_help = false,
    after_long_help = help::template(HELP_DETAIL)
)]
pub struct NodesCommand {}
impl NodesCommand {
    pub fn run (self, api_builder: &mut ApiBuilder, _options: CommandGlobalOpts) {
        api_builder.to_path("nodes".to_string());
    }
}

/// Get details on a specific node or it's resources
#[derive(Clone, Debug, Args)]
#[command(
    after_long_help = help::template(HELP_DETAIL)
)]
pub struct NodeCommand { 
    #[command(flatten)]
    node_opts: NodeOpts,

    #[arg(short, long)]
    detailed: Option<bool>,

    #[command(subcommand)]
    subcommand: Option<NodeSubcommand>
}

#[derive(Clone, Debug, Subcommand)]
pub enum NodeSubcommand {
    Transports(TransportsCommand),
    SecureChannels(SecureChannelsCommand),
}

impl NodeCommand {
    pub fn run(self, api_builder: &mut ApiBuilder, options: CommandGlobalOpts) {
        // Build any actions that will always be relevant on this command and subcommands
        api_builder.to_path("node".to_string());
        api_builder.rpc_to_node(self.node_opts.api_node);

        match self.subcommand {
            Some(subcommand) => {
                match subcommand {
                    NodeSubcommand::Transports(c) => c.run(api_builder, options),
                    NodeSubcommand::SecureChannels(c) => c.run(api_builder, options),
                }
            },
            None => {
                // Build for any specifics within this command
                let payload = GetNodeStatusRequest::new(self.detailed);
                api_builder.exec_with_body(payload, options, print_response)
            }
        }
    }
}

fn print_response(
    rpc: Rpc
) {
    let resp = rpc.parse_response::<NodeStatus>();
    match resp {
        Ok(node_status) => {
            print_node_status(&node_status);
        },
        Err(_) => println!("Error parsing response of node."),
    }
}

fn print_node_status(node_status: &NodeStatus) {
    println!();
    println!("Node:");
    println!("  Name: {}", node_status.node_name);
    println!("  Status: {}", node_status.status.light_green());

    println!("  Route To Node:");
    let mut m = MultiAddr::default();
    if m.push_back(Node::new(node_status.node_name.clone())).is_ok() {
        println!("    Short: {}", m);
    }

    // ! node_port is only available on NodeConfigOld, more work to return from API at the moment
    // TODO: Overseer work
    // let mut m = MultiAddr::default();
    // if m.push_back(DnsAddr::new("localhost")).is_ok() && m.push_back(Tcp::new(node_status.node_port)).is_ok() {
    //     println!("    Verbose: {}", m);
    // }

    match &node_status.details {
        Some(node_details) => print_node_details(&node_details),
        None => ()
    }
}

fn print_node_details(node_details: &NodeDetails) {
    println!("  Identity: {}", String::from(node_details.short_identity.identity_id.clone()));

    println!("  Transports:");
    for e in &node_details.transport_list.list {
        println!("    Transport:");
        println!("      Type: {}", e.tt);
        println!("      Mode: {}", e.tm);
        println!("      Address: {}", e.payload);
    }

    println!("  Secure Channel Listeners:");
    for e in &node_details.secure_channel_listeners {
        println!("    Listener:");
        if let Some(ma) = addr_to_multiaddr(e) {
            println!("      Address: {}", ma);
        }
    }

    println!("  Inlets:");
    for e in &node_details.inlets.list {
        println!("    Inlet:");
        println!("      Listen Address: {}", e.bind_addr);
        if let Some(r) = Route::parse(e.outlet_route.as_ref()) {
            if let Some(ma) = route_to_multiaddr(&r) {
                println!("      Route To Outlet: {}", ma);
            }
        }
    }

    println!("  Outlets:");
    for e in &node_details.outlets.list {
        println!("    Outlet:");
        println!("      Forward Address: {}", e.tcp_addr);

        if let Some(ma) = addr_to_multiaddr(e.worker_addr.as_ref()) {
            println!("      Address: {}", ma);
        }
    }

    println!("  Services:");
    for e in &node_details.services.list {
        println!("    Service:");
        println!("      Type: {}", e.service_type);
        if let Some(ma) = addr_to_multiaddr(e.addr.as_ref()) {
            println!("      Address: {}", ma);
        }
    }
}