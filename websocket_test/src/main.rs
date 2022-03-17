mod ws;
mod messages;
mod lobby;
mod start_connection;
mod login;
mod iota_logic;
mod database;
use actix::Actor;
use actix_web::{web, test::init_service};
use std::env;
use lobby::Lobby;
use start_connection::start_connection as start_connection_route;

use iota_logic::initiate::initiate;
use iota_logic::client::create_client;
use database::init_database;

use actix_web::{App, HttpServer};

#[actix_web::main]
async fn main() -> std::io::Result<()> {

    //database::insert_doctor(String::from("DrHouse"), String::from("230332956"), String::from("hd5f4dsf54aw5d48vc5v256")).unwrap();
    //database::insert_doctor(String::from("DrDorian"), String::from("230415486"), String::from("jgz54he98s61vs2yaf846489")).unwrap();
    //database::insert_doctor(String::from("DrCox"), String::from("230769124"), String::from("tuz4n56cv1nbdf9498f4as65fa98")).unwrap();
    //database::blacklist_doctor(String::from("230332956"));
    

    // type 'cargo run initiate' to initiate or reset the channel
    let arg: Vec<_> = env::args().collect();
    if(arg.len() > 1) {
        if(arg[1] == "initiate") {
            let transport = create_client();
            initiate(transport).unwrap();
        }
    }

    println!("Running Server!");
    let ws_server = Lobby::default().start();

    HttpServer::new(move || {
        App::new()
            .service(start_connection_route)
            .route("/login_registration_office", web::post().to(login::login_registration_office))
            .route("/doctor_first_login", web::post().to(login::first_login))
            .route("/doctor_login", web::post().to(login::doctor_login))
            .route("/put_blacklist", web::post().to(login::put_doctor_on_blacklist))
            .route("/remove_from_blacklist", web::post().to(login::remove_doctor_from_blacklist))
            .route("/certificate", web::post().to(login::upload_certificate))
            .route("/CheckCertificate", web::post().to(login::check_certificate))
            .route("/healthCertificate", web::post().to(login::upload_health_certificate))
            .route("/CheckHealthCertificate", web::post().to(login::check_health_certificate))
            .data(ws_server.clone())
    })
    .bind("192.168.0.202:8080")?
    .run()
    .await
}