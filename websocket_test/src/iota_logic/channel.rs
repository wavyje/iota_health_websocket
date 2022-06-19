use std::{str, vec, collections::LinkedList, convert::{TryFrom, TryInto}};

use std::str::FromStr;

use actix_web::http::header::IntoHeaderValue;
use iota_streams::{app::{message::HasLink, transport::tangle::{TangleAddress}, identifier::Identifier}, app_channels::{api::tangle::{
    Author,
    Subscriber,
    PublicKey
}}, core::{
prelude::Rc,
println
}, ddml::types::*};

use iota_streams::{
app::transport::{
tangle::client::{Client},
},
core::{
prelude::{ String}
},
};
use core::cell::RefCell;
use serde_json::{json, Value};
use rand::Rng;

use iota_streams::app_channels::api::{psk_from_seed, pskid_from_psk, pskid_from_str, Psk, PskId};
use super::super::database;


/// Imports the author instance from file "author_state.bin" and then gets the channel address from file "channel_address.bin" and converts it
/// to a usable TangleAddress.
///
/// Author instance and TangleAddress get returned, as well as a success indicator.
///
/// # Arguments:
///
/// *transport: Rc<Refcell<Client>> - is the application instance of the channel
///
/// *password: &str - password with which the author was exported to the file
#[tokio::main]
pub async fn import_author(transport: Rc<RefCell<Client>>, password: &str) -> (bool, Option<Author<Rc<RefCell<Client>>>>, Option<TangleAddress>){
    

     // Retrieve author state from file
     let state = std::fs::read("./author_state.bin").unwrap();

     // Import author instance
     let author = Author::import(&state, &password, transport).await;

     //catch error
    match author {
        Ok(ref a) => {println!("AppInst: {}", &a.channel_address().unwrap());},
        Err(_e) => return (false, None, None)
    }

    // get channel address
    let ann = std::fs::read("./channel_address.bin").unwrap();
    let ann_str = str::from_utf8(&ann).unwrap();

    //turn address into TangleAddress
    let ann_link_split = ann_str.split(':').collect::<Vec<&str>>();
    let announce_str = String::from(ann_link_split[0]) + ":" + ann_link_split[1];
    let announce_link = TangleAddress::from_str(&announce_str).unwrap();
   
    println!("Address: {}", &announce_link);

    return (true, Some(author.unwrap()), Some(announce_link));
}

/// Imports the subscriber instance from file "subscriber_state.bin" and then gets the channel address from file "channel_address.bin" and converts it
/// to a usable TangleAddress.
///
/// Subscriber instance and TangleAddress get returned, as well as a success indicator.
///
/// # Arguments:
///
/// *transport: Rc<Refcell<Client>> - is the application instance of the channel
/// *password: &str - password with which the author was exported to the file
#[tokio::main]
pub async fn import_subscriber(transport: Rc<RefCell<Client>>, password: &str) -> (bool, Option<Subscriber<Rc<RefCell<Client>>>>, Option<TangleAddress>){
    

     // Retrieve state from file
     let state = std::fs::read("./subscriber_state.bin").unwrap();

     // Import state
     let subscriber = Subscriber::import(&state, &password, transport).await;

     // catch error
    match subscriber {
        Ok(ref a) => {println!("AppInst: {}", &a.channel_address().unwrap());},
        Err(_e) => return (false, None, None)
    }

    // get channel address
    let ann = std::fs::read("./channel_address.bin").unwrap();
    let ann_str = str::from_utf8(&ann).unwrap();

    // turn address into TangleAddress
    let ann_link_split = ann_str.split(':').collect::<Vec<&str>>();
    let announce_str = String::from(ann_link_split[0]) + ":" + ann_link_split[1];
    let announce_link = TangleAddress::from_str(&announce_str).unwrap();
   
    println!("Address: {}", &announce_link);

    return (true, Some(subscriber.unwrap()), Some(announce_link));
}

/// Creates an individual keyload and publishes a signed message on it, containing a hash of the data
///
/// # Arguments
///
/// *data: String - is the hash of the user's data
/// *author: Author<Rc<RefCell<Client>>> - is the imported author instance
/// *announce_link: TangleAddress - usable channel address
/// *password: &str - password with which the author was exported
#[tokio::main]
pub async fn post_registration_certificate(data: String, mut author: Author<Rc<RefCell<Client>>>, announce_link: TangleAddress, password: &str) -> Value {

    let key = rand::thread_rng().gen::<[u8; 32]>();
    
    //transform key into string seed for json
    let mut seed = String::new();

    for i in 0..32 {
        seed.push_str(&key[i].to_string());
        if i<31 {
            seed.push_str(",");
        }
    }
    
    let psk = psk_from_seed(&key);

    let psk_id = pskid_from_psk(&psk);
    //Identifier from PskId is required by new IOTA Streams version
    let i = Identifier::PskId(psk_id);

    // author synchronizing the state
    author.fetch_state().unwrap();
    author.store_psk(psk_id, psk).unwrap();

    println!("{}, ::::: {}", &announce_link, &author.channel_address().unwrap());
    
    // create branch with sub's public key
    let keyload_link = {
        let (_msg, seq) = author.send_keyload(&announce_link, [i].into_iter()).await.unwrap();
        let seq = seq.unwrap();
        seq
    };
    
    // create public payload with hash
    // masked payload is empty
    let public_payload = Bytes(data.as_bytes().to_vec());
    let empty_masked_payload = Bytes("".as_bytes().to_vec());

    // author publishes signed message linked to keyload message
    let signed_message_link = { 
        let (_msg, seq) = author.send_signed_packet(&keyload_link, &public_payload, &empty_masked_payload).await.unwrap(); 
        let seq = seq.unwrap();
        seq
    };

    //put message links in json
    let res_json = json!({"appInst": announce_link.base().to_string(), "AnnounceMsgId": announce_link.msgid.to_string(), "KeyloadMsgId": keyload_link.msgid.to_string(), "SignedMsgId": signed_message_link.msgid.to_string(), "PskSeed": seed});
    
    //export author again with password
    let state = author.export(&password).await.unwrap();
    std::fs::write("./author_state.bin", state).unwrap();

    return res_json;
}

/// Publishes a tagged message linked to the signed message of the specific branch, containing a hash of the data
///
/// # Arguments
///
/// *data: String - is the hash of the user's data
/// *subscriber: Subscriber<Rc<RefCell<Client>>> - is the imported subscriber instance
/// *keyload_link: TangleAddress - usable keyload sequence
/// *signed_msg_link: TangelAddress - usable signed message sequence
/// *password: &str - password with which the subscriber was exported
#[tokio::main]
pub async fn post_health_certificate(data: String, mut subscriber: Subscriber<Rc<RefCell<Client>>>, keyload_link: TangleAddress, signed_msg_link: TangleAddress, password: &str, pskSeed: [u8;32]) -> Value {

    //sub must join the channel

    // get channel address
    let ann = std::fs::read("./channel_address.bin").unwrap();
    let ann_str = str::from_utf8(&ann).unwrap();

    //turn address into TangleAddress
    let ann_link_split = ann_str.split(':').collect::<Vec<&str>>();
    let announce_str = String::from(ann_link_split[0]) + ":" + ann_link_split[1];
    let announce_link = TangleAddress::from_str(&announce_str).unwrap();

    subscriber.receive_announcement(&announce_link).await.unwrap();
    subscriber.send_subscribe(&announce_link).await.unwrap();
    subscriber.sync_state().await.unwrap();
    

    /////////////////////////////

     //retrieve psk
     let psk = psk_from_seed(&pskSeed);
     let psk_id = pskid_from_psk(&psk);

    // subscriber.store_psk(PskId::from_slice(&psk_id).to_owned(), Psk::from_slice(&psk).to_owned()).unwrap();

    // subscriber processing all new messages, so he can find the signed message
    subscriber.sync_state().await.unwrap();

    // receive keyload
    let msg_tag =  subscriber.receive_sequence(&keyload_link).await.unwrap();
    
    let _result = subscriber.receive_keyload(&msg_tag).await;

    // create payload with hash, masked payload is empty
    let public_payload = Bytes(data.as_bytes().to_vec());
    let empty_masked_payload = Bytes("".as_bytes().to_vec());

    // publish tagged message linked to signed message
    let tagged_message_link = { 
        let (_msg, seq) = subscriber.send_tagged_packet(&signed_msg_link, &public_payload, &empty_masked_payload).await.unwrap(); 
        let seq = seq.unwrap();
        seq
    };

    //put message links in json
    let res_json = json!({"Certificate": "health_certificate", "TaggedMsgId": tagged_message_link.msgid.to_string()});
    
    //export subscriber again
    /*let state = subscriber.export(&password).await.unwrap();
    std::fs::write("./subscriber_state.bin", state).unwrap();*/

    return res_json;
}

/// Nameless subscriber joins channel and branch and checks the signed message on it for the public payload.
/// If public paylaod matches the transferred hash, the registration certificate is valid.
///
/// # Arguments
/// *transport: Rc<RefCell<Client>> - client connecting to node
/// *appInst: String - application instance address
/// *announce_link: String - channel address
/// *keyload_link: String - keyload message address
/// *signed_msg_link: String - signed message address
/// root_hash: String - hash of the user's data, was calculated on the mobile device
#[tokio::main]
pub async fn check_registration_certificate(mut subscriber: Subscriber<Rc<RefCell<Client>>>, transport: Rc<RefCell<Client>>, appInst: String, announce_link: String, keyload_link: String, signed_msg_link: String, root_hash: String, pskSeed: [u8; 32]) -> bool {
    
    //retrieve psk
    let psk = psk_from_seed(&pskSeed);
    let psk_id = pskid_from_psk(&psk);


    // get channel address
    let ann = std::fs::read("./channel_address.bin").unwrap();
    let ann_str = str::from_utf8(&ann).unwrap();

    //turn address into TangleAddress
    let ann_link_split = ann_str.split(':').collect::<Vec<&str>>();
    let announce_str = String::from(ann_link_split[0]) + ":" + ann_link_split[1];
    let announce_link = TangleAddress::from_str(&announce_str).unwrap();

    subscriber.receive_announcement(&announce_link).await.unwrap();
    subscriber.send_subscribe(&announce_link).await.unwrap();
    subscriber.store_psk(PskId::from_slice(&psk_id).to_owned(), Psk::from_slice(&psk).to_owned()).unwrap();
    

    // IMPORTANT, OTHERWISE IT WILL NOT FIND ANY MESSAGES
    subscriber.sync_state().await.unwrap();

    println!("Keyload {}", &keyload_link);


    let keyload_str = appInst.clone() + ":" + &keyload_link;

    // receive keyload
    let msg_tag =  subscriber.receive_sequence(&TangleAddress::from_str(&keyload_str).unwrap()).await.unwrap();
    
    let _result = subscriber.receive_keyload(&msg_tag).await;
    
    let signed_str = appInst.clone() + ":" + &signed_msg_link;

    // receive signed message
    let msg_tag = subscriber.receive_sequence(&TangleAddress::from_str(&signed_str).unwrap()).await.unwrap();
    
    let (_signer_pk, unwrapped_public, _) = subscriber.receive_signed_packet(&msg_tag).await.unwrap();
    
    // get public payload
    let unwrapped_public = unwrapped_public.0;
    println!("Public Message: {}", String::from_utf8(unwrapped_public.clone()).unwrap());
    println!("Transferred: {}", String::from(&root_hash));
    
    // unregister subscriber to minimize traffic
    subscriber.unregister();

    // compare payload to root_hash, if equal return true
    if(String::from_utf8(unwrapped_public.clone()).unwrap() == String::from(&root_hash))  {
        return true;
    }
    else {
        return false;
    }
}

/// Nameless subscriber joins channel and branch and checks the tagged message on it for the public payload.
/// If public payload matches the transferred hash, the health certificate is valid.
///
/// # Arguments
/// *transport: Rc<RefCell<Client>> - client connecting to node
/// *appInst: String - application instance address
/// *announce_link: String - channel address
/// *keyload_link: String - keyload message address
/// *tagged_msg_link: String - tagged message address
/// root_hash: String - hash of the user's data, was calculated on the mobile device
#[tokio::main]
pub async fn check_health_certificate(mut subscriber: Subscriber<Rc<RefCell<Client>>>, appInst: String, keyload_link: String, tagged_msg_link: String, root_hash: String, pskSeed: [u8;32]) -> (bool, bool) {
    

    //retrieve psk
    let psk = psk_from_seed(&pskSeed);
    let psk_id = pskid_from_psk(&psk);


    // get channel address
    let ann = std::fs::read("./channel_address.bin").unwrap();
    let ann_str = str::from_utf8(&ann).unwrap();

    //turn address into TangleAddress
    let ann_link_split = ann_str.split(':').collect::<Vec<&str>>();
    let announce_str = String::from(ann_link_split[0]) + ":" + ann_link_split[1];
    let announce_link = TangleAddress::from_str(&announce_str).unwrap();

    subscriber.receive_announcement(&announce_link).await.unwrap();
    subscriber.send_subscribe(&announce_link).await.unwrap();
    subscriber.store_psk(psk_id, psk).unwrap();

    //IMPORTANT, OTHERWISE IT WILL NOT FIND ANY MESSAGES
    subscriber.sync_state().await.unwrap();

    println!("Keyload {}", &keyload_link);

    let keyload_str = appInst.clone() + ":" + &keyload_link;

    // receive keyload
    let msg_tag =  subscriber.receive_sequence(&TangleAddress::from_str(&keyload_str).unwrap()).await.unwrap();
    
    let _result = subscriber.receive_keyload(&msg_tag).await;
    
    subscriber.sync_state().await.unwrap();

    let tagged_str = appInst.clone() + ":" + &tagged_msg_link;

    // receive tagged message
    let msg_tag = subscriber.receive_sequence(&TangleAddress::from_str(&tagged_str).unwrap()).await.unwrap();
    
    let (unwrapped_public, _) = subscriber.receive_tagged_packet(&msg_tag).await.unwrap();
    
    let unwrapped_public = unwrapped_public.0;
    println!("Public Message: {}", String::from_utf8(unwrapped_public.clone()).unwrap());
    println!("Transferred: {}", String::from(&root_hash));

    let payload_string = String::from_utf8(unwrapped_public.clone()).unwrap();
    let payload_vector = payload_string.split(":").collect::<Vec<&str>>();
    let certificate_hash = payload_vector[1];
    let doctor_lanr = payload_vector[0];

    //collects the results of the querys
    //(doctor not on blacklist = true, certificate valid = true)
    let mut result: (bool, bool) = (false, false);
    
    // compare public payload to root_hash, if equal return true
    if certificate_hash == String::from(&root_hash)  {
        result.1 = true;
    }
    
    //check if lanr is on the blacklist
    let blacklist_query = database::search_blacklist(String::from(doctor_lanr));
       
    match blacklist_query {
        Err(e) => {
            result.0 = true;
        },
        Ok(()) => {
            result.0 = false;
        }
    }

    result
    
}