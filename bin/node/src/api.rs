#![feature(proc_macro_hygiene, decl_macro)]

use redstone_rs::*;
use log::*;
use rocket::{routes, Route};
use std::io::prelude::*;
use lazy_static::*;
use serde::Deserialize;
use std::sync::Mutex;
use reqwest::Client;
use std::time::{UNIX_EPOCH, SystemTime};
use std::error::Error;
use rocket::get;
use rocket::post;
use redstone_rs::transaction::Transaction;
use redstone_rs::account::Account;
use redstone_rs::keypair;

use rocket::config::{Config, Environment, LoggingLevel};
lazy_static! {
    static ref WALLET_DETAILS: Mutex<Vec<String>> = Mutex::new(Vec::new());
}
#[derive(Default, Clone, Deserialize, Debug)]
struct TxnDetails {
    pub amount: u64,
    pub reciever: String,
    pub sender: String,
    pub txn_type: String,
    pub extra: String,
}

#[derive(Clone, Deserialize, Debug)]
struct Blockcount {
    success: bool,
    blockcount: u64,
}

#[derive(Clone, Deserialize, Debug)]
struct HashAtHeight {
    success: bool,
    hash: String,
}

#[derive(Clone, Deserialize, Debug)]
struct Transactioncount {
    success: bool,
    transaction_count: u64,
}
#[get("/")]
fn must_provide_method() -> &'static str {
    "{ \"success\": false, \"error\": \"METHOD_MISSING\" }"
}

#[get("/ping")]
fn ping() -> &'static str {
    //output everyting
    "{ \"success\": true, \"response\": \"Pong!\" }"
}


#[post("/submit_txn", format = "application/json", data = "<txn_data>")]
pub fn submit_txn_v1(txn_data: rocket::Data) -> String {
    debug!("Transaction recived by api!");
    let mut holder_vec: Vec<u8> = vec![];
    let mut txn_data1 = txn_data.open();

    loop {
        let mut buffer = [0u8; 512];
        let try_read_from_stream = txn_data1.read(&mut buffer);
        if let Ok(size) = try_read_from_stream {
            trace!("Read {} bytes into buffer", size);
            if size == 0 {
                break;
            } else {
                holder_vec.append(&mut buffer.to_vec());
            }
        } else {
            debug!(
                "Failed to read into buf, error={}",
                try_read_from_stream.unwrap_err()
            );
            return format!(" {{ \"error\" : \" failed to read from datastream \" }}");
        }
    }
    let try_utf8_to_json = String::from_utf8(holder_vec);
    if let Ok(txn_pretrim) = try_utf8_to_json {
        if txn_pretrim != "" {
            let mut txn = txn_pretrim[1..].replace("\\", "").to_string(); // this very verbose bit of code removes everything outside the { } and removes the \
            loop {
                if &txn[txn.len() - 1..] != "}" {
                    txn = txn[0..txn.len() - 1].to_string();
                } else {
                    break;
                }
            }
            trace!("Txn submited by API json={}", txn);
            let try_string_to_txn = serde_json::from_str::<Transaction>(&txn);
            if let Ok(txn1) = try_string_to_txn {
                mempool::add_transaction(txn1);
                return "{ \"result\" : \"sent txn\" }".to_owned();
            } else {
                return "{ \"result\" : \"FAILURE PLEASE TRY LATER\" }".to_owned();
            }
        }
        else {
            debug!(
              "Failed to turn utf8 bytes to txn (submit txn api, error={})",
              "no"
            );
        return format!(" {{ \"error\" : \" utf8 to json failed \" }}");
        }
  }else {
        return "{ \"result\" : \"failure\" }".to_owned();
  }
}

#[get("/get_mem_tx/<hash>")]
fn gettx(hash: String) -> String {
    if let Err(get1) = mempool::get_transaction(hash.clone())  {
        return "{ \"result\" : \"failure\" }".to_owned();    } 
    else {
        let get = serde_json::to_string(&mempool::get_transaction(hash.clone()).unwrap());
        return "{ \"success\": true, \"Result\":".to_string() + &get.unwrap() + "}";

    }
}

#[get("/get_block/<hash>")]
fn get_blk(hash: String) -> String {
    
    // TODO: Get chain count from the config, for every chain look in to the db get block by hash if it exist
    let mut result = "{ \"result\" : \"failure\" }".to_owned();
    todo!();
    return(result.to_string());
}


#[get("/get_acc/<public_key>")]
fn getacc(public_key: String) -> String {
    if let Err(get1) = Account::get(public_key.clone())  {
        return "{ \"result\" : \"failure\" }".to_owned();    } 
    else {
        let get = serde_json::to_string(&Account::get(public_key).unwrap());
        return "{ \"success\": true, \"Result\":".to_string() + &get.unwrap() + "}";
    }
}
#[get("/create_wallet")]
fn create_wallet() -> String {
    let wallet = redstone_rs::keypair::Keypair::generate();
    let public_key = wallet.public_key.to_string();
    let private_key = wallet.private_key.to_string();
    let address = wallet.address().to_string();
    return format!("{{ \"public_key\": \"{}\", \"private_key\": \"{}\", \"address\": \"{}\" }}", public_key, private_key, address);
}


pub fn get_middleware() -> Vec<Route> {
    routes![must_provide_method,
            ping,
            submit_txn_v1,
            gettx,
            getacc,
            create_wallet
    ]
}
pub fn start_api() {
    let config = Config::build(Environment::Staging)
        .log_level(LoggingLevel::Critical) // disables logging
        .finalize()
        .unwrap();

    rocket::custom(config)
        .mount("/json_api/", get_middleware())
        .launch();

}