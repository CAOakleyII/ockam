use std::{
    net::{IpAddr, SocketAddr},
    str::FromStr,
};

use anyhow::{anyhow, Context as _, Result};
use clap::Args;
use ockam::{AsyncTryClone, Context, TcpTransport};
use ockam_api::nodes::{overseer::Overseer, OVERSEER_ADDR};

use crate::{
    help,
    util::{
        bind_to_port_check, embedded_node_that_is_not_stopped, exitcode, find_available_port,
        node_rpc, startup,
    },
    CommandGlobalOpts,
};
// ! Should change to be configurable and have a default option
pub const OVERSEER_NODE_NAME: &str = "overseer";

const HELP_DETAIL: &str = "\
About:
    Initializes an Ockam Environment.

Examples:
```sh
    # Initialize your environment with a default configuration
    $ ockam init
```
";

/// Initializes your environment with the default configurations
/// and spawns an API for Management
#[derive(Clone, Debug, Args)]
#[command(
    arg_required_else_help = false,
    after_long_help = help::template(HELP_DETAIL)
)]
pub struct InitCommand {
    /// TCP listener address
    #[arg(
        display_order = 900,
        long,
        short,
        id = "SOCKET_ADDRESS",
        default_value = "127.0.0.1:0"
    )]
    pub tcp_listener_address: String,

    /// ockam_command started a child process to run this node in foreground.
    #[arg(display_order = 900, long, hide = true)]
    pub init_in_current_process: bool,
}

impl Default for InitCommand {
    fn default() -> Self {
        Self {
            tcp_listener_address: "127.0.0.1:0".to_string(),
            init_in_current_process: false,
        }
    }
}

impl InitCommand {
    pub fn run(self, options: CommandGlobalOpts) {
        if self.init_in_current_process {
            if let Err(e) = create_foreground_node_for_overseer(&options, &self) {
                std::process::exit(e.code());
            }
        } else {
            // create a background process and re-run this command in that process
            // but with the option `init_in_current_process` flipped to true.
            node_rpc(run_impl, (options, self))
        }
    }

    fn overwrite_addr(&self) -> Result<Self> {
        let cmd = self.clone();
        let addr: SocketAddr = if &cmd.tcp_listener_address == "127.0.0.1:0" {
            let port = find_available_port().context("failed to acquire available port")?;
            SocketAddr::new(IpAddr::from_str("127.0.0.1")?, port)
        } else {
            cmd.tcp_listener_address.parse()?
        };
        Ok(Self {
            tcp_listener_address: addr.to_string(),
            ..cmd
        })
    }
}

async fn run_impl(
    ctx: ockam::Context,
    (opts, cmd): (CommandGlobalOpts, InitCommand),
) -> crate::Result<()> {
    let _cfg = &opts.config;
    if cmd.init_in_current_process {
        return Err(crate::Error::new(
            exitcode::CONFIG,
            anyhow!("Cannot create a background node from background node"),
        ));
    }

    // Spawn overseer node in another, new process
    let cmd = cmd.overwrite_addr()?;
    let addr = SocketAddr::from_str(&cmd.tcp_listener_address)?;
    spawn_overseer_node(&ctx, &opts, &cmd, addr).await?;

    println!("Ockam environment has been initialized.");

    Ok(())
}

async fn spawn_overseer_node(
    _ctx: &Context,
    opts: &CommandGlobalOpts,
    cmd: &InitCommand,
    addr: SocketAddr,
) -> crate::Result<()> {
    let verbose = opts.global_args.verbose;
    let cfg = &opts.config;

    // Check if the port is used by some other services or process
    if !bind_to_port_check(&addr) || cfg.port_is_used(addr.port()) {
        return Err(crate::Error::new(
            exitcode::IOERR,
            anyhow!("Another process is listening on the provided port!"),
        ));
    }

    // First we create a new node in the configuration so that
    // we can ask it for the correct log path, as well as
    // making sure the watchdog can do its job later on.
    cfg.create_node(OVERSEER_NODE_NAME, addr, verbose)?;
    cfg.persist_config_updates()?;

    startup::spawn_overseer_node(
        &opts.config,
        verbose,
        OVERSEER_NODE_NAME,
        &cmd.tcp_listener_address,
    )?;

    Ok(())
}

fn create_foreground_node_for_overseer(
    opts: &CommandGlobalOpts,
    cmd: &InitCommand,
) -> crate::Result<()> {
    let verbose = opts.global_args.verbose;
    let node_name = OVERSEER_NODE_NAME;
    let cfg = &opts.config;

    let cmd = cmd.overwrite_addr()?;
    let addr = SocketAddr::from_str(&cmd.tcp_listener_address)?;

    // HACK: try to get the current node dir.  If it doesn't
    // exist the user PROBABLY started a non-detached node.
    // Thus we need to create the node dir so that subsequent
    // calls to it don't fail
    if cfg.get_node_dir(node_name).is_err() {
        cfg.create_node(node_name, addr, verbose)?;
        cfg.persist_config_updates()?;
    }

    embedded_node_that_is_not_stopped(run_foreground_overseer_node, (opts.clone(), cmd, addr))?;
    return Ok(());
}

async fn run_foreground_overseer_node(
    ctx: Context,
    (_opts, cmd, _addr): (CommandGlobalOpts, InitCommand, SocketAddr),
) -> crate::Result<()> {
    let tcp = TcpTransport::create(&ctx).await?;
    let bind = cmd.tcp_listener_address;
    tcp.listen(&bind).await?;

    let overseer_worker = Overseer::new(tcp.async_try_clone().await?);

    ctx.start_worker(OVERSEER_ADDR, overseer_worker).await?;

    Ok(())
}
