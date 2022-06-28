use crate::Options;
use clap::{App, ArgMatches};

pub fn read_cli() -> Options {
    let matches = App::new("lock-unspents")
        .version("0.1.0")
        .author("Kasper Ziemianek <kasper.ziemianek@gmail.com>")
        .about("Manager your btc unspents!")
        .arg(
            Arg::with_name("host")
                .short("h")
                .long("rpc-host")
                .value_name("HOST")
                .help("Node rpc host")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("port")
                .short("p")
                .long("rpc-port")
                .value_name("PORT")
                .help("Node rpc port")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("rpc-user")
                .long("rpc-user")
                .value_name("RPC_USER")
                .help("Node rpc user")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("rpc-password")
                .long("rpc-password")
                .value_name("RPC_PASSWORD")
                .help("Node rpc password")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("omit-tx")
                .long("omit-tx")
                .value_name("TX_TO_OMIT")
                .help("tx id to omit")
                .takes_value(true),
        )
        .get_matches();

    let rpc_host = get_argument(&matches, "host");
    let rpc_port = get_argument(&matches, "port");
    let rpc_user = get_argument(&matches, "rpc-user");
    let rpc_pass = get_argument(&matches, "rpc-password");

    Options {
        rpc_host,
        rpc_port,
        rpc_user,
        rpc_pass,
    }
}

fn get_argument(matches: &ArgMatches, name: &str) -> String {
    matches.value_of(name).unwrap().to_owned()
}
