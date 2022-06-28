use crate::app::Options;
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
pub struct Unspent {
    pub txid: String,
    pub vout: u8,
}

pub fn lock_unspent(unspent: &Unspent, options: &Options) {
    let client = reqwest::blocking::Client::new();
    let node_url = "http://".to_owned() + &options.rpc_host + ":" + &options.rpc_port + "/";

    let mut json = serde_json::Value::default();
    json["txid"] = json!(unspent.txid);
    json["vout"] = json!(unspent.vout);

    let list_unspent = RpcMethod {
        jsonrpc: "1.0".to_owned(),
        id: "lock-unspents".to_owned(),
        method: "lockunspent".to_owned(),
        params: serde_json::Value::Array(vec![json!(false), serde_json::Value::Array(vec![json])]),
    };
    let req = serde_json::to_string(&list_unspent).unwrap();
    client
        .post(&node_url)
        .basic_auth(
            options.rpc_user.to_owned(),
            Some(options.rpc_pass.to_owned()),
        )
        .body(req)
        .send()
        .unwrap();
}

pub fn get_unspents(options: &Options) -> Vec<Unspent> {
    //
    // let list_unspent = RpcMethod {
    //     jsonrpc: "1.0".to_owned(),
    //     id: "lock-unspents".to_owned(),
    //     method: "listunspent".to_owned(),
    //     params: serde_json::Value::Array(vec![
    //         json!(1),
    //         json!(999999),
    //         serde_json::Value::Array(vec![]),
    //         serde_json::Value::Bool(true),
    //     ]),
    // };
    //
    // let req = serde_json::to_string(&list_unspent).unwrap();
    // let client = reqwest::blocking::Client::new();
    // let node_url = "http://".to_owned() + &options.rpc_host + ":" + &options.rpc_port + "/";
    //
    // let resp: serde_json::Value = client
    //     .post(&node_url)
    //     .basic_auth(options.rpc_user.to_owned(), Some(options.rpc_pass.to_owned()))
    //     .body(req)
    //     .send()
    //     .unwrap()
    //     .json()
    //     .unwrap();
    // let unspents: ListUnspentResponse = serde_json::from_value(resp).unwrap();
    // unspents.result

    vec![
        Unspent {
            txid: "123".to_owned(),
            vout: 0,
        },
        Unspent {
            txid: "456".to_owned(),
            vout: 1,
        },
        Unspent {
            txid: "789".to_owned(),
            vout: 0,
        },
    ]
}
