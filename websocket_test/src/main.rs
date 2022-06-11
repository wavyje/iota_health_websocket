
mod ws;
mod messages;
mod lobby;
mod start_connection;
mod login;
mod iota_logic;
mod database;
use actix::Actor;
use actix_web::web;
use std::env;
use std::sync::Arc;
use lobby::Lobby;
use start_connection::start_connection as start_connection_route;

use rustls::{ServerConfig, NoClientAuth};
use rustls_pemfile::certs;
use rustls::Certificate;

use rustls::ResolvesServerCertUsingSNI;
use rustls::sign::{SigningKey, RSASigningKey};

use iota_logic::initiate::initiate;
use iota_logic::client::create_client;

use actix_web::{App, HttpServer};

#[actix_web::main]
async fn main() -> std::io::Result<()> {

    // type 'cargo run initiate' to initiate or reset the channel
    let arg: Vec<_> = env::args().collect();
    if arg.len() > 1 {
        if arg[1] == "initiate" {
            let transport = create_client();
            initiate(transport).unwrap();
        }
    }

    println!("Running Server!");
    let ws_server = Lobby::default().start();

    let config = load_ssl();

    HttpServer::new(move || {
        App::new()
            .service(start_connection_route)
            .route("/login_registration_office", web::post().to(login::login_registration_office))
            .route("/doctor_first_login", web::post().to(login::first_login))
            .route("/doctor_login", web::post().to(login::doctor_login))
            .route("/put_blacklist", web::post().to(login::put_doctor_on_blacklist))
            .route("/remove_from_blacklist", web::post().to(login::remove_doctor_from_blacklist))
            .route("/remove_doctor", web::post().to(login::remove_doctor))
            .route("/certificate", web::post().to(login::upload_certificate))
            .route("/CheckCertificate", web::post().to(login::check_certificate))
            .route("/healthCertificate", web::post().to(login::upload_health_certificate))
            .route("/CheckHealthCertificate", web::post().to(login::check_health_certificate))
            .data(ws_server.clone())
    })
    .bind_rustls("134.106.186:8080", config)?
    .server_hostname("bling02.vlba.uni-oldenburg.de")
    .run()
    .await
}

fn load_ssl() -> rustls::ServerConfig {
    use std::io::BufReader;
 
    const CERT: &'static [u8] = include_bytes!("cert.pem");
    const KEY: &'static [u8] = include_bytes!("key.pem");
 
    let mut cert = BufReader::new(CERT);
    let mut key = BufReader::new(KEY);

    let cert_chain = certs(&mut cert)
        .unwrap()
        .iter()
        .map(|v| Certificate(v.clone()))
        .collect();

    let mut keys = rustls::PrivateKey(Vec::new()); 
    loop {
            match rustls_pemfile::read_one(&mut key).expect("cannot parse private key .pem file") {
                Some(rustls_pemfile::Item::RSAKey(key)) => keys = rustls::PrivateKey(key),
                Some(rustls_pemfile::Item::PKCS8Key(key)) => keys = rustls::PrivateKey(key),
                Some(rustls_pemfile::Item::ECKey(key)) => keys = rustls::PrivateKey(key),
                None => break,
                _ => {}
            }
        }
 
        let mut config = ServerConfig::new(NoClientAuth::new());
        config.set_single_cert(cert_chain, keys).unwrap();

    //connect to hostname
    let mut resolver = ResolvesServerCertUsingSNI::new();

    add_certificate_to_resolver("iota_health", "bling02.vlba.uni-oldenburg.de", &mut resolver);

    config.cert_resolver = Arc::new(resolver);

    config
 
 }

 fn add_certificate_to_resolver(
    name: &str, hostname: &str,
    resolver: &mut ResolvesServerCertUsingSNI
) {
    use std::io::BufReader;
 
    const CERT: &'static [u8] = include_bytes!("cert.pem");
    const KEY: &'static [u8] = include_bytes!("key.pem");
 
    let mut cert = BufReader::new(CERT);
    let mut key = BufReader::new(KEY);

    let cert_chain = certs(&mut cert)
        .unwrap()
        .iter()
        .map(|v| Certificate(v.clone()))
        .collect();

    let mut keys = rustls::PrivateKey(Vec::new()); 
    loop {
            match rustls_pemfile::read_one(&mut key).expect("cannot parse private key .pem file") {
                Some(rustls_pemfile::Item::RSAKey(key)) => keys = rustls::PrivateKey(key),
                Some(rustls_pemfile::Item::PKCS8Key(key)) => keys = rustls::PrivateKey(key),
                Some(rustls_pemfile::Item::ECKey(key)) => keys = rustls::PrivateKey(key),
                None => break,
                _ => {}
            }
        }
    let signing_key = RSASigningKey::new(
        &keys
    ).unwrap();
    
    let signing_key_boxed: Arc<Box<dyn SigningKey>> = Arc::new(
        Box::new(signing_key)
    );

    resolver.add(hostname, rustls::sign::CertifiedKey::new(
        cert_chain, signing_key_boxed
    )).expect("Invalid certificate for corona.ai");
}