use iota_streams::{app::{message::HasLink, transport::tangle::{PAYLOAD_BYTES, TangleAddress, TangleMessage}}, app_channels::{api::tangle::{
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

use core::cell::RefCell;
use rand::Rng;
use rand::AsByteSliceMut;

use std::{collections::hash_map::DefaultHasher};
use std::hash::{Hash, Hasher};
use std::str;

//here channel is created, author instance and subscriber instance created
//
//put the channel adress and encrypted author, subscriber in state
//
//WARNING!!! Only use this function when you are sure the previous channel does not work anymore
// or this is the first time you want to test your system
//TODO: ENV ARGS as input
//TODO: Hash Password
//TODO: Check if passwords are the same, they cant be!
#[tokio::main]
pub async fn initiate(transport: Rc<RefCell<Client>>) {
    /*println!("SIND SIE SICH SICHER?") env
    seedenv, ...env
    auhtor
    */
    println!("Initialisieren");
    let encoding = "utf-8";
    let multi_branching = true;
    let alph9 = "ABCDEFGHIJKLMNOPQRSTUVWXYZ9";
    let seed: &str = &(0..10)
        .map(|_| alph9.chars().nth(rand::thread_rng().gen_range(0, 27)).unwrap())
        .collect::<String>();
    let mut author = Author::new(seed, encoding, PAYLOAD_BYTES, multi_branching, transport.clone());
    println!("Author created!");
    
    //publish channel, print adress to note
    let announce_link = author.send_announce().unwrap();
    println!("Channel published!");
    println!("Important: Note the adress!!! Channel adress: {}", &announce_link);
    println!("Base: {} ; MsgId {}", announce_link.base(), announce_link.msgid);
    
   
   
   
    /*
    //hash password
    let mut hasher = DefaultHasher::new();
    seed.hash(hasher);
    println!Vec
    seedenv, ...env
    subscriber
    export subsc(pwnenv)*/

    //Doctor
    let seed1: &str = &(0..10)
        .map(|_| alph9.chars().nth(rand::thread_rng().gen_range(0, 27)).unwrap())
        .collect::<String>();
    let mut subscriber = Subscriber::new(seed1, encoding, PAYLOAD_BYTES, transport.clone());
    println!("Subscriber created!");

    //subscriber join channel
    subscriber.receive_announcement(&announce_link).unwrap();
    //try_or!(registration_office.get_author().channel_address() == doctor_one.get_subscriber().channel_address(), ApplicationInstanceMismatch(String::from("Channel not matching")))?;

    //subscriber subscribe to channel
    let subscribe_link = subscriber.send_subscribe(&announce_link).unwrap();
    println!("Subscribed to the channel");
    

    // create subscriber instance for reading messages
    let seed2: &str = &(0..10)
        .map(|_| alph9.chars().nth(rand::thread_rng().gen_range(0, 27)).unwrap())
        .collect::<String>();
    let mut subscriber_reading = Subscriber::new(seed2, encoding, PAYLOAD_BYTES, transport.clone());

    //subscriber join channel
    subscriber_reading.receive_announcement(&announce_link).unwrap();

    //subscriber subscribe to channel
    let subscribe_link2 = subscriber_reading.send_subscribe(&announce_link).unwrap();
    println!("Subscribed to the channel");


    author.receive_subscribe(&subscribe_link).unwrap();
    author.receive_subscribe(&subscribe_link2).unwrap();

     //export author
     let state = author.export("Geheimes Passwort").unwrap();
     std::fs::write("./author_state.bin", state).unwrap();
     //println!("Encrypted Author is: {:?}", String::from_utf8(state));
 
     //save channel address
     std::fs::write("./channel_address.bin", &announce_link.to_string()).unwrap();

    //export subscriber, hash password
    let encrypted_subscriber = subscriber.export("abc").unwrap();
    std::fs::write("./subscriber_state.bin", encrypted_subscriber).unwrap();


    let encrypted_subscriber_reading = subscriber.export("").unwrap();
    std::fs::write("./subscriber_reading_state.bin", encrypted_subscriber_reading).unwrap();
}