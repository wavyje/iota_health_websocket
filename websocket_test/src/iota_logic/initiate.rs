use actix_web::web::Buf;
use iota_streams::{app::{message::HasLink, transport::tangle::{PAYLOAD_BYTES}}, app_channels::{api::tangle::{
    Author,
    Subscriber,
}}, core::{
prelude::Rc,
println,
LOCATION_LOG,
try_or,
}};

use iota_streams::{
app::transport::{
tangle::client::{Client},
},
core::{
prelude::{ String},
Result,
Errors::ApplicationInstanceMismatch
},
};

use core::cell::RefCell;
use rand::Rng;

use sha2::{Digest, Sha256};
use std::{str, io};

/// here channel is created, author instance and subscriber instance created
///
/// put the channel adress and encrypted author, subscriber in state
///
/// WARNING!!! Only use this function when you are sure the previous channel does not work anymore
/// or this is the first time you want to test your system
/// TODO: ENV ARGS as input
/// TODO: Hash Password
/// TODO: Check if passwords are the same, they cant be!
#[tokio::main]
pub async fn initiate(transport: Rc<RefCell<Client>>) -> Result<()>{
    
    // Creates author
    println!("Initializing...");
    let encoding = "utf-8";
    let multi_branching = true;
    let alph9 = "ABCDEFGHIJKLMNOPQRSTUVWXYZ9";
    let seed: &str = &(0..10)
        .map(|_| alph9.chars().nth(rand::thread_rng().gen_range(0, 27)).unwrap())
        .collect::<String>();
    let mut author = Author::new(seed, encoding, PAYLOAD_BYTES, multi_branching, transport.clone());

    let mut tmp = String::new();
    
    println!("Please enter password for author(registration office), it must not contain empty spaces!!!");
    io::stdin().read_line(&mut tmp)
        .ok()
        .expect("Couldn't read password");   

    let password_author = tmp.trim_end();
    // hash the password
    let mut hasher = Sha256::new();
        hasher.update(password_author);
        let result = hasher.finalize();
        let result_format: String = format!("{:x}", result);

        print!("*{}*, {}", password_author, result_format);
    
        println!("Author created! Note down the seed, in case the password is lost: {}", seed);
    
    // publish channel, print adress
    let announce_link = author.send_announce()?;
    println!("Channel published!");
    println!("Channel address: {}", &announce_link);
    println!("Base: {} ; MsgId {}", announce_link.base(), announce_link.msgid);

    // Create doctor
    let seed1: &str = &(0..10)
        .map(|_| alph9.chars().nth(rand::thread_rng().gen_range(0, 27)).unwrap())
        .collect::<String>();
    let mut subscriber = Subscriber::new(seed1, encoding, PAYLOAD_BYTES, transport.clone());

    let mut tmp = String::new();

    println!("Please enter password for subscriber(doctor), it must not contain empty spaces!!!");
    io::stdin().read_line(&mut tmp)
        .ok()
        .expect("Couldn't read password");   
    
    let password_subscriber = tmp.trim_end();

    // hash the password
    let mut hasher2 = Sha256::new();
        hasher2.update(&password_subscriber);
        let result2 = hasher2.finalize();
        let result_format2: String = format!("{:x}", result2);
    
        println!("Subscriber created! Note down the seed, in case the password is lost: {}", seed1);

    // subscriber join channel
    subscriber.receive_announcement(&announce_link)?;
    try_or!(author.channel_address() == subscriber.channel_address(), ApplicationInstanceMismatch(String::from("Channel not matching")))?;

    //subscriber subscribe to channel
    let subscribe_link = subscriber.send_subscribe(&announce_link)?;
    println!("Subscribed to the channel");
    

    // create subscriber instance for reading messages
    let seed2: &str = &(0..10)
        .map(|_| alph9.chars().nth(rand::thread_rng().gen_range(0, 27)).unwrap())
        .collect::<String>();
    let mut subscriber_reading = Subscriber::new(seed2, encoding, PAYLOAD_BYTES, transport.clone());

    // subscriber join channel
    subscriber_reading.receive_announcement(&announce_link)?;

    //subscriber subscribe to channel
    let subscribe_link2 = subscriber_reading.send_subscribe(&announce_link)?;
    println!("Subscribed to the channel");


    author.receive_subscribe(&subscribe_link)?;
    author.receive_subscribe(&subscribe_link2)?;

     //export author
     println!("{}", &result_format);
     let state = author.export(&result_format)?;
     std::fs::write("./author_state.bin", state)?;
 
     //save channel address
     std::fs::write("./channel_address.bin", &announce_link.to_string())?;

    //export subscriber, hash password
    let encrypted_subscriber = subscriber.export(&result_format2)?;
    std::fs::write("./subscriber_state.bin", encrypted_subscriber)?;


    let encrypted_subscriber_reading = subscriber.export("")?;
    std::fs::write("./subscriber_reading_state.bin", encrypted_subscriber_reading)?;

    println!("All instances exported, channel was created");

    Ok(())
}