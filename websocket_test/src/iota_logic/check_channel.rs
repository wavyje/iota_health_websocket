use std::{hash::Hash, str::FromStr, vec};

use actix_web::web::Json;
use iota_streams::{app::{message::HasLink, transport::tangle::{PAYLOAD_BYTES, TangleAddress, TangleMessage, MsgId, AppInst}}, app_channels::{api::tangle::{
    Author,
    Subscriber,
    PublicKey
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
use crate::iota_logic::client;
use serde_json::json;
use serde_json::Value;
use super::stats;
use rand::Rng;
use std::str;

#[tokio::main]
pub async fn importauthor(transport: Rc<RefCell<Client>>, password: &str) -> (bool, Option<Author<Rc<RefCell<Client>>>>, Option<TangleAddress>){
    
    //get static values for channel, author, subscriber
    let stats = Stats::default();
    
    //let mut encrypted_author: Vec<u8> = stats.encrypted_author;
    //let a = encrypted_author.as_byte_slice_mut();

    //let author = Author::import(a, password, transport.clone()); 

     // Retrieve State from file
     let state = std::fs::read("./author_state.bin").unwrap();

     // Import state
     let mut author = Author::import(&state, &password, transport);

    match author {
        Ok(ref a) => {println!("AppInst: {}", &a.channel_address().unwrap());},
        Err(_e) => return (false, None, None)
    }
    

    /*let app = AppInst::from_str(&stats.announce_link_base).unwrap();
    let msg = MsgId::from_str(&stats.announce_link_msg).unwrap();
    let announce_link = TangleAddress::from_base_rel(&app, &msg);*/

    let ann = std::fs::read("./channel_address.bin").unwrap();
    let ann_str = str::from_utf8(&ann).unwrap();

    let ann_link_split = ann_str.split(':').collect::<Vec<&str>>();
    let announce_link = TangleAddress::from_str(ann_link_split[0], ann_link_split[1]).unwrap();
   
    println!("Address: {}", &announce_link);

    return (true, Some(author.unwrap()), Some(announce_link));
}

#[tokio::main]
pub async fn register_certificate(data: String, mut author: Author<Rc<RefCell<Client>>>, announce_link: TangleAddress, password: &str) -> Value {
    //create subscriber, so that different keyloads are created
    //previous subscribers would have access to new branches, but subscriber instances are dropped immediately after sending keyload

    
    
    let encoding = "utf-8";
    let alph9 = "ABCDEFGHIJKLMNOPQRSTUVWXYZ9";
    let seed: &str = &(0..10)
        .map(|_| alph9.chars().nth(rand::thread_rng().gen_range(0, 27)).unwrap())
        .collect::<String>();
        println!("{}",seed);

    let mut subscriber = Subscriber::new(seed, encoding, PAYLOAD_BYTES, author.get_transport().clone());
    let sub_pk = vec![PublicKey::from_bytes(subscriber.get_pk().as_bytes()).unwrap()];

    author.fetch_state().unwrap();

    println!("{}, ::::: {}", &announce_link, &author.channel_address().unwrap());
    
    //create branch
    let keyload_link = {
        let (msg, seq) = author.send_keyload(&announce_link, &[], &sub_pk).unwrap();
        let seq = seq.unwrap();
        seq
    };
    //YJYFHSBZBM
    //create Payload
    let public_payload = Bytes(data.as_bytes().to_vec());
    let empty_masked_payload = Bytes("".as_bytes().to_vec());

    let signed_message_link = { 
        let (msg, seq) = author.send_signed_packet(&keyload_link, &public_payload, &empty_masked_payload).unwrap(); 
        let seq = seq.unwrap();
        seq
    };

    //put links in json
    let res_json = json!({"appInst": announce_link.base().to_string(), "AnnounceMsgId": announce_link.msgid.to_string(), "KeyloadMsgId": keyload_link.msgid.to_string(), "SignedMsgId": signed_message_link.msgid.to_string()});
    
    //export author again
    let state = author.export(&password).unwrap();
    std::fs::write("./author_state.bin", state).unwrap();

    return res_json;
}

pub async fn crypt(data: String) {
    
}
//check if channel is still valid

#[tokio::main]
pub async fn check_certificate(transport: Rc<RefCell<Client>>, appInst: String, announce_link: String, keyload_link: String, signed_msg_link: String, root_hash: String) -> bool {
    
    //create subscriber instance
    
    println!("Tranpsort");
    let encoding = "utf-8";
    let alph9 = "ABCDEFGHIJKLMNOPQRSTUVWXYZ9";
    let seed: &str = &(0..10)
        .map(|_| alph9.chars().nth(rand::thread_rng().gen_range(0, 27)).unwrap())
        .collect::<String>();

    let mut subscriber = Subscriber::new(seed, encoding, PAYLOAD_BYTES, transport);
    println!("Subciebr");
    let announce = subscriber.receive_announcement(&TangleAddress::from_str(&appInst, &announce_link).unwrap());
    println!("Announce");
    /*match announce {
        Err(_e) => return false,
        Ok(()) => ()
    }*/
        
    println!("AnnpounceMatch");
    //receive keyload
    
    let msg_tag =  subscriber.receive_sequence(&TangleAddress::from_str(&appInst, &keyload_link).unwrap()).unwrap();
    
    let result = subscriber.receive_keyload(&msg_tag);
    

    //receive signed message
    
    let msg_tag = subscriber.receive_sequence(&TangleAddress::from_str(&appInst, &signed_msg_link).unwrap()).unwrap();
    
    let (_signer_pk, unwrapped_public, unwrapped_masked) = subscriber.receive_signed_packet(&msg_tag).unwrap();
    
    let unwrapped_public = unwrapped_public.0;
    println!("Public Message: {}", String::from_utf8(unwrapped_public.clone()).unwrap());
    println!("Transferred: {}", String::from(&root_hash));
    
    if(String::from_utf8(unwrapped_public.clone()).unwrap() == String::from(&root_hash))  {
        println!("Es hat funktioniert");
        return true;
    }
    else {
        return false;
    }
}