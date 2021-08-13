use std::{str, vec};

use iota_streams::{app::{message::HasLink, transport::tangle::{PAYLOAD_BYTES, TangleAddress}}, app_channels::{api::tangle::{
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
     let author = Author::import(&state, &password, transport);

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
    let announce_link = TangleAddress::from_str(ann_link_split[0], ann_link_split[1]).unwrap();
   
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
     let subscriber = Subscriber::import(&state, &password, transport);

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
    let announce_link = TangleAddress::from_str(ann_link_split[0], ann_link_split[1]).unwrap();
   
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
    
    // create subscriber, so that different keyloads are created
    // subscriber instances are dropped immediately after sending keyload
    let encoding = "utf-8";
    let alph9 = "ABCDEFGHIJKLMNOPQRSTUVWXYZ9";
    let seed: &str = &(0..10)
        .map(|_| alph9.chars().nth(rand::thread_rng().gen_range(0, 27)).unwrap())
        .collect::<String>();
        println!("{}",seed);

    // get subscriber's public key
    let subscriber = Subscriber::new(seed, encoding, PAYLOAD_BYTES, author.get_transport().clone());
    let sub_pk = vec![PublicKey::from_bytes(subscriber.get_pk().as_bytes()).unwrap()];

    // author synchronizing the state
    author.fetch_state().unwrap();

    println!("{}, ::::: {}", &announce_link, &author.channel_address().unwrap());
    
    // create branch with sub's public key
    let keyload_link = {
        let (_msg, seq) = author.send_keyload(&announce_link, &[], &sub_pk).unwrap();
        let seq = seq.unwrap();
        seq
    };
    
    // create public payload with hash
    // masked payload is empty
    let public_payload = Bytes(data.as_bytes().to_vec());
    let empty_masked_payload = Bytes("".as_bytes().to_vec());

    // author publishes signed message linked to keyload message
    let signed_message_link = { 
        let (_msg, seq) = author.send_signed_packet(&keyload_link, &public_payload, &empty_masked_payload).unwrap(); 
        let seq = seq.unwrap();
        seq
    };

    //put message links in json
    let res_json = json!({"appInst": announce_link.base().to_string(), "AnnounceMsgId": announce_link.msgid.to_string(), "KeyloadMsgId": keyload_link.msgid.to_string(), "SignedMsgId": signed_message_link.msgid.to_string()});
    
    //export author again with password
    let state = author.export(&password).unwrap();
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
pub async fn post_health_certificate(data: String, mut subscriber: Subscriber<Rc<RefCell<Client>>>, keyload_link: TangleAddress, signed_msg_link: TangleAddress, password: &str) -> Value {

    // subscriber processing all new messages, so he can find the signed message
    subscriber.fetch_all_next_msgs();

    // receive keyload
    let msg_tag =  subscriber.receive_sequence(&keyload_link).unwrap();
    
    let _result = subscriber.receive_keyload(&msg_tag);

    // create payload with hash, masked payload is empty
    let public_payload = Bytes(data.as_bytes().to_vec());
    let empty_masked_payload = Bytes("".as_bytes().to_vec());

    // publish tagged message linked to signed message
    let tagged_message_link = { 
        let (_msg, seq) = subscriber.send_tagged_packet(&signed_msg_link, &public_payload, &empty_masked_payload).unwrap(); 
        let seq = seq.unwrap();
        seq
    };

    //put message links in json
    let res_json = json!({"Certificate": "health_certificate", "TaggedMsgId": tagged_message_link.msgid.to_string()});
    
    //export subscriber again
    let state = subscriber.export(&password).unwrap();
    std::fs::write("./subscriber_state.bin", state).unwrap();

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
pub async fn check_registration_certificate(mut subscriber: Subscriber<Rc<RefCell<Client>>>, transport: Rc<RefCell<Client>>, appInst: String, announce_link: String, keyload_link: String, signed_msg_link: String, root_hash: String) -> bool {
    
    // create subscriber instance
    /*let encoding = "utf-8";
    let alph9 = "ABCDEFGHIJKLMNOPQRSTUVWXYZ9";
    let seed: &str = &(0..10)
        .map(|_| alph9.chars().nth(rand::thread_rng().gen_range(0, 27)).unwrap())
        .collect::<String>();

    // subscribe to channel
    let mut subscriber = Subscriber::new(seed, encoding, PAYLOAD_BYTES, transport);
    let _announce = subscriber.receive_announcement(&TangleAddress::from_str(&appInst, &announce_link).unwrap());*/

    // IMPORTANT, OTHERWISE IT WILL NOT FIND ANY MESSAGES
    subscriber.fetch_all_next_msgs();

    println!("Keyload {}", &keyload_link);

    // receive keyload
    let msg_tag =  subscriber.receive_sequence(&TangleAddress::from_str(&appInst, &keyload_link).unwrap()).unwrap();
    
    let _result = subscriber.receive_keyload(&msg_tag);
    

    // receive signed message
    let msg_tag = subscriber.receive_sequence(&TangleAddress::from_str(&appInst, &signed_msg_link).unwrap()).unwrap();
    
    let (_signer_pk, unwrapped_public, _) = subscriber.receive_signed_packet(&msg_tag).unwrap();
    
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
pub async fn check_health_certificate(mut subscriber: Subscriber<Rc<RefCell<Client>>>, appInst: String, keyload_link: String, tagged_msg_link: String, root_hash: String) -> bool {
    
    //IMPORTANT, OTHERWISE IT WILL NOT FIND ANY MESSAGES
    subscriber.fetch_all_next_msgs();

    println!("Keyload {}", &keyload_link);

    // receive keyload
    let msg_tag =  subscriber.receive_sequence(&TangleAddress::from_str(&appInst, &keyload_link).unwrap()).unwrap();
    
    let _result = subscriber.receive_keyload(&msg_tag);

    // receive signed message
    let msg_tag = subscriber.receive_sequence(&TangleAddress::from_str(&appInst, &tagged_msg_link).unwrap()).unwrap();
    
    let (unwrapped_public, _) = subscriber.receive_tagged_packet(&msg_tag).unwrap();
    
    let unwrapped_public = unwrapped_public.0;
    println!("Public Message: {}", String::from_utf8(unwrapped_public.clone()).unwrap());
    println!("Transferred: {}", String::from(&root_hash));
    
    // compare public payload to root_hash, if equal return true
    if(String::from_utf8(unwrapped_public.clone()).unwrap() == String::from(&root_hash))  {
        return true;
    }
    else {
        return false;
    }
}