mod ws;
mod messages;
mod lobby;
mod start_connection;
mod login;
mod iota_logic;
use actix::Actor;
use actix_web::web;
use lobby::Lobby;
use start_connection::start_connection as start_connection_route;
use login::login;
use iota_logic::initiate::initiate;
use iota_logic::client::create_client;
use iota_logic::check_channel::importauthor;
use login::upload_certificate;

use actix_web::{App, HttpServer};

#[actix_web::main]
async fn main() -> std::io::Result<()> {

    //For more information on this visit README->chapter: 1) Getting started
    //WARNING: Only uncomment the following lines, if you know what you are doing!
    //let transport = create_client();
    //initiate(transport);
    //importauthor(transport, "Geheimes Passwort");

    let chat_server = Lobby::default().start();

    HttpServer::new(move || {
        App::new()
            .service(start_connection_route)
            .route("/login", web::post().to(login::login))
            .route("/certificate", web::post().to(login::upload_certificate))
            .data(chat_server.clone())
    })
    .bind("192.168.0.202:8080")?
    .run()
    .await
}