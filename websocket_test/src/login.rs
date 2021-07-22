
use std::rc::Rc;

use actix_web::{self, App, Error, HttpMessage, HttpRequest, HttpResponse, HttpServer, Result, http::{self, StatusCode}, post, web::{self, Form}};
use iota_streams::app_channels::api::tangle::{Author, Subscriber};
use iota_streams::app::transport::tangle::{PAYLOAD_BYTES, TangleAddress};
use rand::AsByteSliceMut;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use rand::Rng;

use crate::iota_logic::{check_channel::{self, register_certificate}, client};
use crate::iota_logic::check_channel::importauthor;
use crate::iota_logic::merkle_tree::generate_merkle_tree;

#[derive(Deserialize)]
pub struct FormData {
    password: String,
}

pub async fn login(form: web::Form<FormData>) -> Result<HttpResponse, Error>{
    
    println!("{}", form.password);
    let transport = client::create_client();

    let (success, author, _) = importauthor(transport, &form.password);

    match success {
        true => Ok(HttpResponse::Ok().body("correctPassword")),
        false => Ok(HttpResponse::Ok().status(StatusCode::FORBIDDEN).finish())
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
    
    //create Strng of data
    /*let mut req_string = String::from("");
    req_string.push_str(&form.firstName);
    req_string.push_str(&form.lastName);
    req_string.push_str(&form.birthday);
    req_string.push_str(&form.birthplace);
    req_string.push_str(&form.nationality);
    req_string.push_str(&form.address);
    req_string.push_str(&form.hashedImage);
    req_string.push_str(&form.expire);

    println!("reqString: {}", req_string);

    //hash data
    let mut hasher = Sha256::new();
    hasher.update(req_string);
    let mut result = hasher.finalize();
    let rootHash: String = format!("{:x}", result);

    println!("{:x}", result);*/

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
    
    let transport = client::create_client();

    let result = check_channel::check_certificate(transport, form.appInst.clone(), form.AnnounceMsgId.clone(), form.KeyloadMsgId.clone(), form.SignedMsgId.clone(), form.rootHash.clone());

    match result {
        true => return Ok(HttpResponse::Ok().finish()),
        false => return Ok(HttpResponse::Ok().status(StatusCode::FORBIDDEN).finish())
    }

    
}