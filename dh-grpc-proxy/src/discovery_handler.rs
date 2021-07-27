use akri_discovery_utils::discovery::{
    discovery_handler::{deserialize_discovery_details, DISCOVERED_DEVICES_CHANNEL_CAPACITY},
    v0::{discovery_handler_server::DiscoveryHandler, DiscoverRequest, DiscoverResponse},
    DiscoverStream,
};

use super::marshallers::discover_response_marshaller;
use async_trait::async_trait;
use log::{error, info};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::delay_for;
use tonic::{Response, Status};

pub const DISCOVERY_INTERVAL_SECS: u64 = 4;

// Input and output files dir.
pub const OUTPUT_FILE_PATH: &str = "/tmp/wde-dir/out.out";
pub const INPUT_FILE_PATH: &str = "/tmp/wde-dir/in.in";
pub const AVAILABILITY_FILE_PATH: &str = "/tmp/wde-dir/debug-echo-availability.txt";

pub const ONLINE: &str = "ONLINE";
pub const OFFLINE: &str = "OFFLINE";

/// DebugEchoDiscoveryDetails describes the necessary information needed to discover and filter debug echo devices.
/// Specifically, it contains a list (`descriptions`) of fake devices to be discovered.
/// This information is expected to be serialized in the discovery details map sent during Discover requests.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DebugEchoDiscoveryDetails {
    pub descriptions: Vec<String>,
}

pub struct DiscoveryHandlerImpl {
    register_sender: tokio::sync::mpsc::Sender<()>,
}

impl DiscoveryHandlerImpl {
    pub fn new(register_sender: tokio::sync::mpsc::Sender<()>) -> Self {
        DiscoveryHandlerImpl { register_sender }
    }
}

#[async_trait]
impl DiscoveryHandler for DiscoveryHandlerImpl {
    type DiscoverStream = DiscoverStream;
    async fn discover(
        &self,
        request: tonic::Request<DiscoverRequest>,
    ) -> Result<Response<Self::DiscoverStream>, Status> {
        info!("Connection established!");
        let register_sender = self.register_sender.clone();
        let discover_request = request.get_ref();
        let (mut discovered_devices_sender, discovered_devices_receiver) =
            mpsc::channel(DISCOVERED_DEVICES_CHANNEL_CAPACITY);
        let discovery_handler_config: DebugEchoDiscoveryDetails =
            deserialize_discovery_details(&discover_request.discovery_details)
                .map_err(|e| tonic::Status::new(tonic::Code::InvalidArgument, format!("{}", e)))?;

        // Write to input file the Agents request
        write_input_file(discovery_handler_config);
        write_availability_file(ONLINE);

        tokio::spawn(async move {
            loop {
                delay_for(Duration::from_secs(DISCOVERY_INTERVAL_SECS)).await;

                // Check if the output exists.
                if !has_output() {
                    continue;
                }

                let response: DiscoverResponse = read_output_file();
                if let Err(e) = discovered_devices_sender.send(Ok(response)).await {
                    // TODO: consider re-registering here
                    error!(
                        "discover - proxy failed to send discovery response with error {}",
                        e
                    );
                    /*
                    if let Some(mut sender) = register_sender {
                        sender.send(()).await.unwrap();
                    }
                    */
                    break;
                }
            }
        });
        // write_availability_file(OFFLINE);
        Ok(Response::new(discovered_devices_receiver))
    }
}

// This serialize the Agents input and writes it into the input file.
pub fn write_input_file(debug_echo_discovery_details: DebugEchoDiscoveryDetails) {
    let path = Path::new(INPUT_FILE_PATH);

    //TODO: handle errors
    let json_output = serde_json::to_string(&debug_echo_discovery_details).unwrap();
    info!("Input file written: {}", json_output);

    fs::write(path, json_output).expect("Failed to write input!");
}

// This reads the output files and serialize it to the agent gRPC format.
// The file is deleted after this call.
pub fn read_output_file() -> DiscoverResponse {
    let path = Path::new(OUTPUT_FILE_PATH);
    let display = path.display();

    let contents = fs::read_to_string(path).expect(format!("could not read {}", display).as_str());
    info!("Checked for output file and found:\n{}", contents);

    let discovery_handler_config: DiscoverResponse =
        discover_response_marshaller::from_json_to_discover_response(&contents);

    // Delete file.
    fs::remove_file(path).expect("Failed to delete output file!");

    return discovery_handler_config;
}

pub fn write_availability_file(text: &str) {
    let path = Path::new(AVAILABILITY_FILE_PATH);
    fs::write(path, text).expect("Failed to write availability!");
}

// Check if output file has already been printed by the Wasi application.
pub fn has_output() -> bool {
    let path = Path::new(OUTPUT_FILE_PATH);
    return path.exists();
}
