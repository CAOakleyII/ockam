use clap::Args;
use cli_table::{print_stdout, Cell, Style, Table};
use ockam_api::nodes::models::transport::{
    GetTransportList, TransportList, TransportMode, TransportStatus, TransportType,
};

use crate::{
    help,
    util::{api_builder::ApiBuilder, Rpc},
    CommandGlobalOpts,
};

const HELP_DETAIL: &str = "\
About:
    Transports
    ------

    Transports are plugins to the Ockam Routing layer that allow Ockam Routing messages
    to travel across nodes over transport layer protocols like TCP, UDP, BLUETOOTH etc.
";
#[derive(Clone, Debug, Args)]
#[command(
    after_long_help = help::template(HELP_DETAIL)
)]
pub struct TransportsCommand {
    /// Comma seperated list of transport types to list
    /// [TCP, BLE, WebSocket(WS)]
    #[arg(long, value_delimiter = ',')]
    tts: Option<Vec<TransportType>>,

    /// Comma seperated list of transport modes to list
    /// [Listen, Connect]
    #[arg(long, value_delimiter = ',')]
    tms: Option<Vec<TransportMode>>,
}

impl TransportsCommand {
    pub fn run(self, api_builder: &mut ApiBuilder, options: CommandGlobalOpts) {
        let payload = GetTransportList::new(self.tts, self.tms);

        api_builder
            .to_path("transports".to_string())
            .exec_with_body(payload, options, print_response);
    }
}

fn print_response(rpc: Rpc) {
    let resp = rpc.parse_response::<TransportList>();
    match resp {
        Ok(transport_list) => {
            if print_transport_list(&transport_list).is_err() {
                println!("Error outputing the results to stdout.")
            }
        }
        Err(e) => println!("Error getting list of transports: {:?}", e),
    }
}

fn print_transport_list(transport_list: &TransportList) -> crate::Result<()> {
    let table = transport_list
        .list
        .iter()
        .fold(
            vec![],
            |mut acc,
             TransportStatus {
                 tt,
                 tm,
                 payload,
                 tid,
                 ..
             }| {
                let row = vec![tid.cell(), tt.cell(), tm.cell(), payload.cell()];
                acc.push(row);
                acc
            },
        )
        .table()
        .title(vec![
            "Transport ID".cell().bold(true),
            "Transport Type".cell().bold(true),
            "Mode".cell().bold(true),
            "Address bind".cell().bold(true),
        ]);

    print_stdout(table)?;

    Ok(())
}