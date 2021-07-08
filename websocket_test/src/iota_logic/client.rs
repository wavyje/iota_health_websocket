use iota_streams::{
    app::{
        message::HasLink,
        transport::tangle::PAYLOAD_BYTES,
    },
    app_channels::{
        api::tangle
    },
    core::{
        prelude::Rc}
};

use iota_streams::{
    app::transport::{
        TransportOptions,
        tangle::client::{SendOptions, Client, },
    },
};




use core::cell::RefCell;


#[tokio::main]
pub async fn create_client() -> Rc<RefCell<Client>>{
    let node_url = "https://api.lb-0.testnet.chrysalis2.com";

    // Creates Client
    
    let client = Client::new_from_url(&node_url);

    let mut transport = Rc::new(RefCell::new(client));
    let send_opt = SendOptions::default();
    transport.set_send_options(send_opt);

    return transport;
}