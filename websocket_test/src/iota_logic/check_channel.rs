use std::{hash::Hash, str::FromStr, vec};

use actix_web::web::Json;
use iota_streams::{app::{message::HasLink, transport::tangle::{PAYLOAD_BYTES, TangleAddress, TangleMessage, MsgId, AppInst}}, app_channels::{api::tangle::{
    Author,
    Subscriber,
}, message::announce}, core::{
prelude::Rc,
print,
println,
try_or,
LOCATION_LOG,
Errors::*,
}, ddml::types::*};

use iota_streams::{
app::transport::{
TransportOptions,
tangle::client::{SendOptions, Client, },
},
app_channels::api::tangle::Transport,
core::{
prelude::{ String},
Result,
},
};
use rand::AsByteSliceMut;
use core::cell::RefCell;
use crate::iota_logic::stats::Stats;
use serde_json::json;
use serde_json::Value;
use super::stats;

#[tokio::main]
pub async fn importauthor(transport: Rc<RefCell<Client>>, password: &str) -> (bool, Option<Author<Rc<RefCell<Client>>>>, Option<TangleAddress>){
    
    //get static values for channel, author, subscriber
    let stats = Stats::default();
    
    let mut encrypted_author: Vec<u8> = stats.encrypted_author;
    let a = encrypted_author.as_byte_slice_mut();

    let author = Author::import(a, password, transport.clone()); 

    match author {
        Ok(ref a) => {println!("AppInst: {}", &a.channel_address().unwrap());},
        Err(_e) => return (false, None, None)
    }
    

    let app = AppInst::from_str(&stats.announce_link_base).unwrap();
    let msg = MsgId::from_str(&stats.announce_link_msg).unwrap();
    let announce_link = TangleAddress::from_base_rel(&app, &msg);
   
    println!("Address: {}", &announce_link);

    return (true, Some(author.unwrap()), Some(announce_link));
}

#[tokio::main]
pub async fn register_certificate(data: Vec<u8>, mut author: Author<Rc<RefCell<Client>>>, announce_link: TangleAddress) -> Value {
    //create subscriber, so that different keyloads are created
    //previous subscribers would have access to new branches, but subscriber instances are dropped immediately after sending keyload
    
    //create branch
    let keyload_link = {
        let (msg, seq) = author.send_keyload_for_everyone(&announce_link).unwrap();
        let seq = seq.unwrap();
        seq
    };

    //create Payload
    let public_payload = Bytes(data);
    let empty_masked_payload = Bytes("".as_bytes().to_vec());

    let signed_message_link = { 
        let (msg, seq) = author.send_signed_packet(&keyload_link, &public_payload, &empty_masked_payload).unwrap(); 
        let seq = seq.unwrap();
        seq
    };

    //put links in json
    let res_json = json!({"appInst": announce_link.base().to_string(), "AnnounceMsgId": announce_link.msgid.to_string(), "KeyloadMsgId": keyload_link.msgid.to_string(), "SignedMsgId": signed_message_link.msgid.to_string()});
    
    return res_json;
}

pub async fn crypt(data: String) {
    
}
//check if channel is still valid