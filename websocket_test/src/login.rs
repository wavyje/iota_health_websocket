

use actix::actors::resolver::ResolverError;
use actix_web::{self, Error, HttpResponse, Result, http::{StatusCode}, web::{self}};
use iota_streams::{app_channels::api::{tangle::{Subscriber, MsgId}, psk_from_seed, pskid_from_psk}, app::transport::tangle::AppInst};
use iota_streams::app::transport::tangle::{TangleAddress};
use serde::Deserialize;
use rand::Rng;

use crate::iota_logic::{channel::{self, import_subscriber, post_registration_certificate, post_health_certificate}, client};
use crate::iota_logic::channel::import_author;
use crate::iota_logic::merkle_tree::generate_merkle_tree;

use std::str::FromStr;

use crate::database;

#[derive(Deserialize)]
pub struct FormData {
    password: String,
}

/// tries to import author instance with sent password.
/// if successful, a HttpResponse with body "office" is sent.
/// if failed, tries to import subscriber with password.
/// if successful, HttpResponse with body "doctor".
/// if failed HttpResponse with StatusCode 403.
/// 
pub async fn login_registration_office(form: web::Form<FormData>) -> Result<HttpResponse, Error>{
    
    println!("{}", form.password);
    let transport = client::create_client();

    let (success, _author, _) = import_author(transport.clone(), &form.password);

    match success {
        true => Ok(HttpResponse::Ok().body("office")),
        false => Ok(HttpResponse::Ok().status(StatusCode::FORBIDDEN).finish())
            }
}


#[derive(Deserialize)]
pub struct DoctorLogin {
    password: String,
    lanr: String,
}
/// method exists to extract the doctor login from the registration office login
/// because another doctor specific password is used
/// querys doctor row from database and checks blacklist
/// *arguments:
/// - lanr
/// - password
pub async fn doctor_login(form: web::Form<DoctorLogin>) -> Result<HttpResponse, Error> {
    
    let success = database::login(form.lanr.clone(), form.password.clone());

    let check_blacklist = database::search_blacklist(form.lanr.clone());

    match check_blacklist {
        Ok(()) => Ok(HttpResponse::BadRequest().finish()),
        Err(e) => {
            match success {
                Ok(()) => Ok(HttpResponse::Ok().finish()),
                Err(e) => Ok(HttpResponse::Forbidden().finish())
            }
        }
    }

    
}

#[derive(Deserialize)]
pub struct DoctorRegistration {
    name: String,
    lanr: String,
    password: String
}

/// method for registering new doctors in the database
/// authorizes to post health certificates
/// must provide:
/// - lanr
/// - name
/// - password
pub async fn first_login(form: web::Form<DoctorRegistration>) -> Result<HttpResponse, Error> {
    
    // check prüfziffer
    // alternating times 4; times 9
    // sum %10
    // result - 10 = prüfungsziffer
    // (Difference == 10 -> prüfungsziffer == 0)
    // source: https://de.wikipedia.org/wiki/Lebenslange_Arztnummer
    let lanr = form.lanr.clone();
    println!("{}",lanr);

    if lanr.len() != 9 {
        return Ok(HttpResponse::BadRequest().finish());
    }

    let lanr_vec: Vec<_> = lanr.split_terminator("").skip(1).collect();

    let mut lanr_as_int: Vec<i32> = vec!();

    for i in lanr_vec {

        let success = i.parse::<i32>();

        match success {
            Err(_e) => {return Ok(HttpResponse::BadRequest().finish());}
            Ok(res) => {
                lanr_as_int.push(res);
            }
        }
    }

    let mut sum = 0;

    for i in 0..6 {
        println!("{}", lanr_as_int[i]);
        if i%2 == 0 {
            sum += lanr_as_int[i]*4;
        }
        else {
            sum += lanr_as_int[i]*9;
        }
    }

    sum = sum%10;

    println!("{}",sum);

    //if the checking number (index 6 in lanr) is unequal to the result, then lanr is invalid
    if !(10-sum).eq(&lanr_as_int[6]) && sum != 10{
        return Ok(HttpResponse::BadRequest().finish());
    }
    //if the result is 10 and the checking number is not 0, the lanr is invalid
    if !&lanr_as_int[6].eq(&0) && sum == 10 {
        return Ok(HttpResponse::BadRequest().finish());
    }

    
    let exist_query = database::search_doctor(form.lanr.clone());

    match exist_query {
        Ok(()) => Ok(HttpResponse::Conflict().finish()),
        Err(_e) => {
            let success = database::insert_doctor(form.name.clone(), form.lanr.clone(), form.password.clone());

            match success {
                Err(_e) => Ok(HttpResponse::BadRequest().finish()),
                Ok(()) => Ok(HttpResponse::Ok().finish())
            }
        }
    }

}

#[derive(Deserialize)]
pub struct DoctorBlacklist {
    lanr: String
}
/// method for blacklisting a faulty doctor
/// event will be published on the main branch for public protocolling purposes
/// *arguments:
/// - lanr
/// - author_password
pub async fn put_doctor_on_blacklist(form: web::Form<DoctorBlacklist>) -> Result<HttpResponse, Error> {

    let success = database::blacklist_doctor(form.lanr.clone());

    match success {
        true => Ok(HttpResponse::Ok().finish()),
        false => {Ok(HttpResponse::Forbidden().finish())}
    }
    
}

pub async fn remove_doctor_from_blacklist(form: web::Form<DoctorBlacklist>) -> Result<HttpResponse, Error> {

    println!("HERE");
    let success = database::remove_doctor_from_blacklist(form.lanr.clone());

    match success {
        Ok(()) => Ok(HttpResponse::Ok().finish()),
        Err(_e) => Ok(HttpResponse::BadRequest().finish())
    }
}

pub async fn remove_doctor(form: web::Form<DoctorBlacklist>) -> Result<HttpResponse, Error> {

    println!("{}", form.lanr.clone());
    let success = database::remove_doctor(form.lanr.clone());

    match success {
        Ok(()) => Ok(HttpResponse::Ok().finish()),
        Err(_e) => Ok(HttpResponse::BadRequest().finish())
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

/// First creates the root hash with the prostitute's data.
/// Then imports author and calls post_registration_certificate.
/// If successful, HttpResponse with message links is sent.
/// If failed HttpResponse 403
pub async fn upload_certificate(form: web::Form<Data>) -> Result<HttpResponse, Error>{
    
    let root_hash = generate_merkle_tree(form.firstName.clone(), form.lastName.clone(), form.birthday.clone(), form.birthplace.clone(), form.nationality.clone(), form.address.clone(), form.hashedImage.clone(), form.expire.clone());
    
    //create author instance
    let transport = client::create_client();

    let (success, author, announce_link) = import_author(transport, &form.password); 

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
            let link_json = post_registration_certificate(root_hash, auth, announce, &form.password);
            

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
    SignedMsgId: String,
    PskSeed: String,
    lanr: String,
    appInst: String
}

/// First creates the root hash with the prostitute's data.
/// Then imports subscriber and calls post_health_certificate.
/// If successful, HttpResponse with message links is sent.
/// If failed HttpResponse 403
pub async fn upload_health_certificate(form: web::Form<DoctorData>) -> Result<HttpResponse, Error>{

    //Query for doctor
    let login = database::search_blacklist(form.lanr.clone());
    
    println!("SignedbeimUpload {}", &form.SignedMsgId);
    let root_hash = generate_merkle_tree(form.firstName.clone(), form.lastName.clone(), form.birthday.clone(), form.birthplace.clone(), form.nationality.clone(), form.address.clone(), form.hashedImage.clone(), form.expire.clone());
    
    //create payload (lanr:root_hash)
    let payload = form.lanr.clone() + ":" + &root_hash;

    //create author instance
    let transport = client::create_client();

    let seed = form.PskSeed.split(",").collect::<Vec<&str>>();
    
    let mut Psk: [u8;32] = [0; 32];

    for i in 0..32 {
        Psk[i] = seed[i].parse::<u8>().unwrap();
    }


    /*let (success, subscriber, announce_link) = import_subscriber(transport, &form.password); 

    let sub;
    let _announce;

    if(success == true) {
        sub = subscriber.unwrap();
        _announce = announce_link.unwrap();
    }
    else {
        return Ok(HttpResponse::Ok().status(StatusCode::FORBIDDEN).finish());
    }*/
    let alph9 = "ABCDEFGHIJKLMNOPQRSTUVWXYZ9";
    let seed: &str = &(0..10)
    .map(|_| alph9.chars().nth(rand::thread_rng().gen_range(0, 27)).unwrap())
    .collect::<String>();
    //TODO: check if subscriber exists!

    let sub = Subscriber::new(&seed, transport.clone());

    
    println!("sml: {}, kml: {}", form.SignedMsgId, form.KeyloadMsgId);

    let keyload_str = form.appInst.clone() + ":" + &form.KeyloadMsgId;
    let signed_msg_str = form.appInst.clone() + ":" + &form.SignedMsgId;

    let keyload_link = TangleAddress::from_str(&keyload_str).unwrap();
    let signed_msg_link = TangleAddress::from_str(&signed_msg_str).unwrap();
    
    
            let link_json = post_health_certificate(payload, sub, keyload_link, signed_msg_link, &form.password, Psk);
            println!("{}",link_json);

            Ok(HttpResponse::Ok().body(link_json))
       
}


#[derive(Deserialize)]
pub struct CheckData {
    rootHash: String,
    appInst: String,
    AnnounceMsgId: String,
    KeyloadMsgId: String,
    SignedMsgId: String,
    PskSeed: String
}

/// Calls the check_registration_certificate function.
/// Returns either HttpResponse 200 or 403.
pub async fn check_certificate(form: web::Form<CheckData>) -> Result<HttpResponse, Error> {

    println!("Checking");
    println!("SignedMessgsa: {}", &form.SignedMsgId);
    
    let transport = client::create_client();

    let seed = form.PskSeed.split(",").collect::<Vec<&str>>();
    
    let mut Psk: [u8;32] = [0; 32];

    for i in 0..32 {
        Psk[i] = seed[i].parse::<u8>().unwrap();
    }

    
    // Retrieve State from file
    //let state = std::fs::read("./subscriber_reading_state.bin").unwrap();

    // Import state
    //let subscriber = Subscriber::import(&state, "", transport.clone()).unwrap();
    let subscriber = Subscriber::new("", transport.clone());

    let result = channel::check_registration_certificate(subscriber, transport,form.appInst.clone(),form.AnnounceMsgId.clone(),form.KeyloadMsgId.clone(),form.SignedMsgId.clone(),form.rootHash.clone(),Psk);

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
    TaggedMsgId: String,
    PskSeed: String
}

/// Imports the reading subscriber, otherwise the tagged message can not be found.
/// Calls the check_health_certificate function.
/// Returns either HttpResponse 200 or 403
pub async fn check_health_certificate(form: web::Form<CheckHealthData>) -> Result<HttpResponse, Error> {

    //db query and comparison

    println!("TaggedeMessage: {}", &form.TaggedMsgId);
    
    let transport = client::create_client();

    //retrieve pre shared key
    let seed = form.PskSeed.split(",").collect::<Vec<&str>>();
    
    let mut Psk: [u8;32] = [0; 32];

    for i in 0..32 {
        Psk[i] = seed[i].parse::<u8>().unwrap();
    }

    // create new subscriber to read
    let subscriber = Subscriber::new("swadawdsadgbc", transport.clone());

    let result = channel::check_health_certificate(subscriber, form.appInst.clone(), form.KeyloadMsgId.clone(), form.TaggedMsgId.clone(), form.rootHash.clone(), Psk);

    match result {
        //doctor is not blacklisted and the certificate hash equals the calculated hash of the customer
        (true, true) => return Ok(HttpResponse::Ok().finish()),
        //doctor is blacklisted, but certificate hash is correct
        (false, true) => return Ok(HttpResponse::Ok().status(StatusCode::BAD_REQUEST).finish()),
        //doctor is not blacklisted, but certificate hash is incorrect
        (true, false) => return Ok(HttpResponse::Ok().status(StatusCode::CONFLICT).finish()),
        //doctor is blacklisted and certificate hash is incorrect
        (false, false) => return Ok(HttpResponse::Ok().status(StatusCode::FORBIDDEN).finish())
    }

    
}