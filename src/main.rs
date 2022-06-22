use clap::{App, Arg, ArgMatches};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Serialize, Deserialize)]
struct RpcMethod {
    jsonrpc: String,
    id: String,
    method: String,
    params: serde_json::Value,
}

#[derive(Serialize, Deserialize)]
struct ListUnspentResponse {
    result: Vec<Unspent>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Unspent {
    txid: String,
    vout: u8,
}

fn main() {
    let matches = App::new("lock-unspents")
        .version("0.1.0")
        .author("Kasper Ziemianek <kasper.ziemianek@gmail.com>")
        .about("Manager your btc unspents!")
        .arg(
            Arg::with_name("host")
                .short("h")
                .long("host")
                .value_name("HOST")
                .help("Node rpc host")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("port")
                .short("p")
                .long("port")
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
    // let tx_to_omit = get_argument(&matches, "omit-tx");

    let list_unspent = RpcMethod {
        jsonrpc: "1.0".to_owned(),
        id: "lock-unspents".to_owned(),
        method: "listunspent".to_owned(),
        params: serde_json::Value::Array(vec![
            json!(1),
            json!(999999),
            serde_json::Value::Array(vec![]),
            serde_json::Value::Bool(true),
        ]),
    };

    let req = serde_json::to_string(&list_unspent).unwrap();
    let client = reqwest::blocking::Client::new();
    let node_url = "http://".to_owned() + &rpc_host + ":" + &rpc_port + "/";

    let resp: serde_json::Value = client
        .post(&node_url)
        .basic_auth(rpc_user.to_owned(), Some(rpc_pass.to_owned()))
        .body(req)
        .send()
        .unwrap()
        .json()
        .unwrap();
    let unspents: ListUnspentResponse = serde_json::from_value(resp).unwrap();

    println!("Before locking unspents:");
    println!("{:?}", unspents.result);

    for unspent in unspents.result {
        // if unspent.txid == tx_to_omit {
        //     continue;
        // }

        let mut json = serde_json::Value::default();
        json["txid"] = json!(unspent.txid);
        json["vout"] = json!(unspent.vout);

        let list_unspent = RpcMethod {
            jsonrpc: "1.0".to_owned(),
            id: "lock-unspents".to_owned(),
            method: "lockunspent".to_owned(),
            params: serde_json::Value::Array(vec![
                json!(false),
                serde_json::Value::Array(vec![json]),
            ]),
        };
        let req = serde_json::to_string(&list_unspent).unwrap();
        client
            .post(&node_url)
            .basic_auth(rpc_user.to_owned(), Some(rpc_pass.to_owned()))
            .body(req)
            .send()
            .unwrap();
    }

    let req = serde_json::to_string(&list_unspent).unwrap();
    let resp: serde_json::Value = client
        .post(&node_url)
        .basic_auth(rpc_user.to_owned(), Some(rpc_pass.to_owned()))
        .body(req)
        .send()
        .unwrap()
        .json()
        .unwrap();
    let unspents: ListUnspentResponse = serde_json::from_value(resp).unwrap();

    println!("After locking unspents:");
    println!("{:?}", unspents.result);
}

fn get_argument(matches: &ArgMatches, name: &str) -> String {
    matches.value_of(name).unwrap().to_owned()
}
