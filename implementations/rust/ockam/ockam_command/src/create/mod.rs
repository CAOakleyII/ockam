mod node;

use clap::{Args, Subcommand};
use ockam_core::api::Method;
use crate::{help, CommandGlobalOpts, util::api_builder::ApiBuilder};

use self::node::NodeCommand;

const HELP_DETAIL: &str = "\
About:
    Creates a new resources within the environment.

Examples:
```sh
    # Creates a new node named relay
    $ ockam create node relay
```
";

/// Create a resource
#[derive(Clone, Debug, Args)]
#[command(
    arg_required_else_help = false,
    after_long_help = help::template(HELP_DETAIL)
)]
pub struct CreateCommand {
    #[command(subcommand)]
    subcommand: CreateSubcommand,
}

#[derive(Clone, Debug, Subcommand)]
pub enum CreateSubcommand {
    Node(NodeCommand),
}

/// Command to create a resource
impl CreateCommand { 
    pub fn run(self, options: CommandGlobalOpts) {
        let mut api_builder = ApiBuilder::new(Method::Post);

        match self.subcommand {
            CreateSubcommand::Node(c) => c.run(&mut api_builder, options)
        }
    }
}