mod discovery_handler;
mod marshallers;
mod discovery_support;

use akri_discovery_utils::discovery::discovery_handler::run_discovery_handler;
use discovery_handler::DiscoveryHandlerImpl;

use discovery_support::onvif_discover::util;
use std::fs;
use std::path::Path;
use tokio::time::Duration;

#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate yaserde_derive;

pub const DISCOVERY_HANDLER_NAME_LABEL: &str = "DISCOVERY_HANDLER_NAME";
pub const ONVIF_URLS_FILE_PATH: &str = "/tmp/wonvif-dir/onvif-urls.txt";

pub fn get_discovery_handler_name() -> String {
    std::env::var(DISCOVERY_HANDLER_NAME_LABEL).unwrap()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    // Specify the name of this DiscoveryHandler. A discovery handler is usually, but not necessarily, identified by
    // the protocol it uses.
    let name = &get_discovery_handler_name();

    // Temp: get the onvif urls
    write_onvif_url_file().await;

    println!("gRPC proxy running named as: {}!", name);
    // Specify whether the devices discovered by this discovery handler are locally attached (or embedded) to nodes or are
    // network based and usable/sharable by multiple nodes.
    let shared = true;
    // A DiscoveryHandler must handle the Agent dropping a connection due to a Configuration that utilizes this
    // DiscoveryHandler being deleted or the Agent erroring. It is impossible to determine the cause of the
    // disconnection, so in case the Agent did error out, the Discovery Handler should try to re-register.
    let (register_sender, register_receiver) = tokio::sync::mpsc::channel(2);
    // Create a DiscoveryHandler
    let discovery_handler = DiscoveryHandlerImpl::new(register_sender);
    // This function will register the DiscoveryHandler with the Agent's registration socket
    // and serve its discover service over UDS at the socket path
    // `format!("{}/{}.sock"), env::var("DISCOVERY_HANDLERS_DIRECTORY"), name)`.
    println!("Turning the server on!");
    run_discovery_handler(discovery_handler, register_receiver, name, shared).await?;

    Ok(())
}

pub async fn write_onvif_url_file() {
    let path = Path::new(ONVIF_URLS_FILE_PATH);
    println!("Entered onvif url finder");
    let urls = util::simple_onvif_discover(Duration::from_secs(5)).await.unwrap();
    let text = format!("{:?}", urls);
    println!("Found device urls: {}", text);
    fs::write(path, text).expect("Failed to write urls!");
}
