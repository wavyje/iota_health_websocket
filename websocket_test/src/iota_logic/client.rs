use iota_streams::core::prelude::Rc;
use iota_streams::{
    app::transport::{
        TransportOptions,
        tangle::client::{SendOptions, Client, },
    },
};

use core::cell::RefCell;

/// Creates a client that will be used as a transport medium for [author, subscriber].
///
/// As a node the Chrysalis testnet will be used, but can be exchanged for own node or different net:
#[tokio::main]
pub async fn create_client() -> Rc<RefCell<Client>>{
    let node_url = "https://chrysalis-nodes.iota.org";

    // Creates Client
    let client = Client::new_from_url(&node_url);

    let mut transport = Rc::new(RefCell::new(client));
    let send_opt = SendOptions::default();
    transport.set_send_options(send_opt);

    return transport;
}