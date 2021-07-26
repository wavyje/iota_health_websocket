
use std::rc::Rc;

use actix_web::{self, App, Error, HttpMessage, HttpRequest, HttpResponse, HttpServer, Result, dev::Transform, http::{self, StatusCode}, post, web::{self, Form}};
use iota_streams::app_channels::api::tangle::{Author, Subscriber};
use iota_streams::app::transport::tangle::{PAYLOAD_BYTES, TangleAddress};
use rand::AsByteSliceMut;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use rand::Rng;

use crate::iota_logic::{check_channel::{self, import_subscriber, register_certificate, register_health_certificate}, client};
use crate::iota_logic::check_channel::importauthor;
use crate::iota_logic::merkle_tree::generate_merkle_tree;

#[derive(Deserialize)]
pub struct FormData {
    password: String,
}

pub async fn login(form: web::Form<FormData>) -> Result<HttpResponse, Error>{
    
    println!("{}", form.password);
    let transport = client::create_client();

    let (success, author, _) = importauthor(transport.clone(), &form.password);

    match success {
        true => Ok(HttpResponse::Ok().body("office")),
        false => {
            let (succ, subscriber, _) = import_subscriber(transport, &form.password);
            match succ {
                true => Ok(HttpResponse::Ok().body("doctor")),
                false => Ok(HttpResponse::Ok().status(StatusCode::FORBIDDEN).finish())
            }
        }
    }
}

#[derive(Deserialize)]
pub struct Data {
    password: String,
    firstName: String,
    lastName: String,
    birthday: String,
    birthplace: String,
    nationality: String,
    address: String,
    hashedImage: String,
    expire: String
}

pub async fn upload_certificate(form: web::Form<Data>) -> Result<HttpResponse, Error>{
    

    let root_hash = generate_merkle_tree(form.firstName.clone(), form.lastName.clone(), form.birthday.clone(), form.birthplace.clone(), form.nationality.clone(), form.address.clone(), form.hashedImage.clone(), form.expire.clone());
    
    //create author instance
    let transport = client::create_client();

    let (success, author, announce_link) = importauthor(transport, &form.password); 

    let auth;
    let announce;;

    if(success == true) {
        auth = author.unwrap();
        announce = announce_link.unwrap();
    }
    else {
        return Ok(HttpResponse::Ok().status(StatusCode::FORBIDDEN).finish());
    }

    match success {
        true => {
            let link_json = register_certificate(root_hash, auth, announce, &form.password);
            

            Ok(HttpResponse::Ok().body(link_json))
        },
        false => Ok(HttpResponse::Ok().status(StatusCode::FORBIDDEN).finish())
    }
}

#[derive(Deserialize)]
pub struct DoctorData {
    password: String,
    firstName: String,
    lastName: String,
    birthday: String,
    birthplace: String,
    nationality: String,
    address: String,
    hashedImage: String,
    expire: String,
    KeyloadMsgId: String,
    SignedMsgId: String
}

pub async fn upload_health_certificate(form: web::Form<DoctorData>) -> Result<HttpResponse, Error>{
    
    println!("SignedbeimUpload {}", &form.SignedMsgId);
    let root_hash = generate_merkle_tree(form.firstName.clone(), form.lastName.clone(), form.birthday.clone(), form.birthplace.clone(), form.nationality.clone(), form.address.clone(), form.hashedImage.clone(), form.expire.clone());
    
    //create author instance
    let transport = client::create_client();


    let (success, subscriber, announce_link) = import_subscriber(transport, &form.password); 

    let sub;
    let announce;;

    if(success == true) {
        sub = subscriber.unwrap();
        announce = announce_link.unwrap();
    }
    else {
        return Ok(HttpResponse::Ok().status(StatusCode::FORBIDDEN).finish());
    }

    println!("sml: {}, kml: {}", form.SignedMsgId, form.KeyloadMsgId);
    let appinst = sub.channel_address().unwrap().to_string();
    let keyload_link = TangleAddress::from_str(&appinst, &form.KeyloadMsgId).unwrap();
    let signed_msg_link = TangleAddress::from_str(&appinst, &form.SignedMsgId).unwrap();
    
    match success {
        true => {
            let link_json = register_health_certificate(root_hash, sub, keyload_link, signed_msg_link, &form.password);
            println!("{}",link_json);

            Ok(HttpResponse::Ok().body(link_json))
        },
        false => Ok(HttpResponse::Ok().status(StatusCode::FORBIDDEN).finish())
    }
}

/*let auth = author.unwrap();
                    let announce_link = announce_link.unwrap();
                    let link_json = register_certificate(hash_as_vec, auth, announce_link);

                    Ok(HttpResponse::Ok().body(link_json))*/

#[derive(Deserialize)]
pub struct CheckData {
    rootHash: String,
    appInst: String,
    AnnounceMsgId: String,
    KeyloadMsgId: String,
    SignedMsgId: String
}

pub async fn check_certificate(form: web::Form<CheckData>) -> Result<HttpResponse, Error> {

    println!("Checking");
    println!("SignedMessgsa: {}", &form.SignedMsgId);
    
    let transport = client::create_client();

    let result = check_channel::check_certificate(transport, form.appInst.clone(), form.AnnounceMsgId.clone(), form.KeyloadMsgId.clone(), form.SignedMsgId.clone(), form.rootHash.clone());

    match result {
        true => return Ok(HttpResponse::Ok().finish()),
        false => return Ok(HttpResponse::Ok().status(StatusCode::FORBIDDEN).finish())
    }

    
}

#[derive(Deserialize)]
pub struct CheckHealthData {
    rootHash: String,
    appInst: String,
    AnnounceMsgId: String,
    KeyloadMsgId: String,
    TaggedMsgId: String
}

pub async fn check_health_certificate(form: web::Form<CheckHealthData>) -> Result<HttpResponse, Error> {

    println!("Checking");
    println!("TaggedeMessgsa: {}", &form.TaggedMsgId);
    
    let transport = client::create_client();


    // Retrieve State from file
    let state = std::fs::read("./subscriber_reading_state.bin").unwrap();

    // Import state
    let subscriber = Subscriber::import(&state, "", transport.clone()).unwrap();

    let result = check_channel::check_health_certificate(subscriber, transport, form.appInst.clone(), form.AnnounceMsgId.clone(), form.KeyloadMsgId.clone(), form.TaggedMsgId.clone(), form.rootHash.clone());

    match result {
        true => return Ok(HttpResponse::Ok().finish()),
        false => return Ok(HttpResponse::Ok().status(StatusCode::FORBIDDEN).finish())
    }

    
}