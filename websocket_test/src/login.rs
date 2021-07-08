
use actix_web::{self, App, Error, HttpMessage, HttpRequest, HttpResponse, HttpServer, Result, http::{self, StatusCode}, post, web::{self, Form}};
use iota_streams::app_channels::api::tangle::Author;
use rand::AsByteSliceMut;
use serde::Deserialize;
use sha2::{Digest, Sha256};


use crate::iota_logic::{check_channel::register_certificate, client};
use crate::iota_logic::check_channel::importauthor;

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
    name: String,
    address: String,
    birthday: String,
    birthplace: String
}

pub async fn upload_certificate(form: web::Form<Data>) -> Result<HttpResponse, Error>{
    
    //create Strng of data
    let mut req_string = String::from("");
    req_string.push_str(&form.name);
    req_string.push_str(&form.address);
    req_string.push_str(&form.birthday);
    req_string.push_str(&form.birthplace);

    println!("reqString: {}", req_string);

    //hash data
    let mut hasher = Sha256::new();
    hasher.update(req_string);
    let mut result = hasher.finalize();
    let hash_as_vec = result.as_byte_slice_mut().to_vec();

    println!("{:x}", result);

    //create author instance
    let transport = client::create_client();

    let (success, author, announce_link) = importauthor(transport, &form.password); 

    let auth;
    let announce;

    if(success == true) {
        auth = author.unwrap();
        announce = announce_link.unwrap();
    }
    else {
        return Ok(HttpResponse::Ok().status(StatusCode::FORBIDDEN).finish());
    }

    match success {
        true => {
            let link_json = register_certificate(hash_as_vec, auth, announce);
            Ok(HttpResponse::Ok().body(link_json))
        },
        false => Ok(HttpResponse::Ok().status(StatusCode::FORBIDDEN).finish())
    }
}

/*let auth = author.unwrap();
                    let announce_link = announce_link.unwrap();
                    let link_json = register_certificate(hash_as_vec, auth, announce_link);

                    Ok(HttpResponse::Ok().body(link_json))*/