use std::borrow::Cow;

use clap::Args;
use cli_table::{Table, Cell, Style, print_stdout};
use ockam::route;
use ockam_abac::ParseError;
use ockam_api::{nodes::models::secure_channel::{SecureChannelList, ShowSecureChannelResponse}, route_to_multiaddr};

use crate::{CommandGlobalOpts, util::{api_builder::ApiBuilder, Rpc}, help, error::Error};

const HELP_DETAIL: &str ="\
About:
    Secure Channels
    ------

    Secure Channels provide end-to-end encrypted and mutually authenticated communication
    that is safe against eavesdropping, tampering, and forgery of messages en-route.

    To create a secure channel, we first need a secure channel listener. Every node that
    is started with ockam command, by convention, starts a secure channel listener at the
    address /service/api.
";
#[derive(Clone, Debug, Args)]
#[command(
    after_long_help = help::template(HELP_DETAIL)
)]
pub struct SecureChannelsCommand { }

impl SecureChannelsCommand {
    pub fn run(self, api_builder: &mut ApiBuilder, options: CommandGlobalOpts) {
        api_builder
            .to_path("secure_channels".to_string())
            .exec(options, print_response)
    }
}

fn print_response(
    rpc: Rpc
) {
    let resp = rpc.parse_response::<SecureChannelList>();
    match resp {
        Ok(secure_channel_list) => {
            if print_secure_channel_list(&secure_channel_list).is_err() {
                println!("Error outputing the results to stdout")
            }
        }, 
        Err(_) => println!("Error parsing the list of secure channels.")
    }
}

fn print_secure_channel_list(
    secure_channel_list: &SecureChannelList
) -> crate::Result<()> {
    let table = secure_channel_list
        .list
        .iter()
        .fold(
            vec![],
            |mut acc,
                ShowSecureChannelResponse {
                    channel,
                    route,
                    from,
                    ..
                }| {
                // node name = from
                let to = match route {
                    Some(r) => {
                        match parse_secure_channel_route(r) {
                            Ok(parsed_route) => parsed_route,
                            Err(_) => "Error Retrieving".to_string()
                        }
                    },
                    None => "Unknown!".to_string()
                };

                let at = match channel {
                    Some(c) => {
                        match parse_channel_address(c.to_string()) {
                            Ok(parsed_channel) => parsed_channel,
                            Err(_) => "Error Retrieving".to_string()
                        }
                    },
                    None => "Unknown!".to_string()
                };

                let row = vec![from.cell(), to.cell(), at.cell()];
                acc.push(row);
                acc
            },
        )
        .table()
        .title(vec![
            "From".cell().bold(true),
            "To".cell().bold(true),
            "At".cell().bold(true),
        ]);

    print_stdout(table)?;

    Ok(())
}

fn parse_secure_channel_route(r: &Cow<str>) -> Result<String, String> {
    let parts: Vec<&str> = r.split(" => ").collect();
    if parts.len() != 2 {
        return Err(format!("Invalid route received"));
    }

    let r1 = &route![*parts.first().unwrap()];
    let r2 = &route![*parts.get(1).unwrap()];
    let ma1 = route_to_multiaddr(r1)
        .ok_or(format!("Failed to convert route {} to multi-address", r1))?;
    let ma2 = route_to_multiaddr(r2)
        .ok_or(format!("Failed to convert route {} to multi-address", r2))?;

    Ok(format!("{}{}", ma1, ma2))
}

fn parse_channel_address(channel_address: String) -> Result<String, String> {
    let channel_route = &route![channel_address];
    let channel_multiaddr = route_to_multiaddr(channel_route).ok_or(format!(
        "Failed to convert route {} to multi-address",
        channel_route
    ))?;
    Ok(channel_multiaddr.to_string())
}