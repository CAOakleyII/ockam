pub mod node;

pub(crate) use node::{NodeCommand, NodesCommand};
use clap::{Args, Subcommand};
use ockam_core::api::Method;
use crate::{help, CommandGlobalOpts, util::{api_builder::ApiBuilder}};

const HELP_DETAIL: &str = "\
About:
    Gets details of resources within the environment.

Examples:
```sh
    # Get a list of nodes
    $ ockam get nodes

    # Get a node named `relay`
    $ ockam get node -n relay 

    # Get a list of transports within a specific node named `relay`
    $ ockam get node transports -n relay
```
";

/// Gets details of resources within the environment
#[derive(Clone, Debug, Args)]
#[command(
    arg_required_else_help = false,
    after_long_help = help::template(HELP_DETAIL)
)]
pub struct GetCommand {
    #[command(subcommand)]
    subcommand: GetSubcommand,
}

/// Gets details of resources within the environment
#[derive(Clone, Debug, Subcommand)]
pub enum GetSubcommand {
    Nodes(NodesCommand),
    Node(NodeCommand),
}

impl GetCommand { 
    pub fn run(self, options: CommandGlobalOpts) {
        let mut api_builder = ApiBuilder::new(Method::Get);

        match self.subcommand {
            GetSubcommand::Nodes(c) => c.run(&mut api_builder, options),
            GetSubcommand::Node(c) => c.run(&mut api_builder, options)
        }
    }
}