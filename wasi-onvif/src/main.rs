mod discovery;
mod models;

use discovery::discovery_utils::{
    OnvifQuery, OnvifQueryImpl, ONVIF_DEVICE_IP_ADDRESS_LABEL_ID,
    ONVIF_DEVICE_MAC_ADDRESS_LABEL_ID, ONVIF_DEVICE_SERVICE_URL_LABEL_ID,
};

use discovery::discovery_impl::util;
use models::device::Device;
use models::request::{OnvifDiscoveryDetails, OnvifResult};

use log::{error, info, trace};
use std::path::Path;
use std::thread;
use std::time::Duration;
use std::fs;

#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate yaserde_derive;

// Input and output files dir.
pub const OUTPUT_FILE_PATH: &str = "/tmp/wonvif-dir/out.out";
pub const INPUT_FILE_PATH: &str = "/tmp/wonvif-dir/in.in";
pub const DEBUG_FILE_PATH: &str = "/tmp/wonvif-dir/debug.txt";

pub const DISCOVERY_INTERVAL_SECS: u64 = 5;

fn main() {
    // Input variables
    let mut input: OnvifDiscoveryDetails;
    println!("Wasi Onvif Discovery Handler running! :)");
    write_debug_file(DEBUG_FILE_PATH, "Hello World!");

    loop {
        thread::sleep(Duration::from_secs(DISCOVERY_INTERVAL_SECS));

        if !has_input() {
            println!("Input not specified yet!");
            continue;
        }
        input = read_input_file();

        trace!("discover - filters:{:?}", &input);
        let discovered_onvif_cameras = util::simple_onvif_discover();

        trace!("discover - got back with:{:?}", &discovered_onvif_cameras,);
        break;
    }
}

// This reads the input file and serialize it to the proper struct format.
pub fn read_input_file() -> OnvifDiscoveryDetails {
    let path = Path::new(INPUT_FILE_PATH);
    let display = path.display();

    let contents = fs::read_to_string(path).expect(format!("could not read {}", display).as_str());

    let new_details: OnvifDiscoveryDetails = match deserialize_discovery_details(&contents) {
        Ok(details) => details,
        Err(error) => {
            println!("An error ocorred while serializing the input: {:?}", error);
            OnvifDiscoveryDetails {
                ip_addresses: Option::None,
                mac_addresses: Option::None,
                scopes: Option::None,
                discovery_timeout_seconds: -1,
            }
        }
    };

    return new_details;
}

// This received the device list and output it in the proper JSON format to the
// output file.
pub fn write_output_file(_devices: Vec<Device>) {
    let path = Path::new(OUTPUT_FILE_PATH);

    // Write output values on DebugEchoResult
    let output_obj: OnvifResult = OnvifResult { devices: _devices };

    //TODO: handle errors
    let json_output = serde_json::to_string(&output_obj).unwrap();
    println!("output: {}", json_output);

    fs::write(path, json_output).expect("Failed to write output!");
}

pub fn write_debug_file(file_path: &str, value: &str) {
    let path = Path::new(file_path);

    fs::write(path, value).expect("Failed to write debug!");
}

// Check if input file has already been sent by gRPC proxy.
pub fn has_input() -> bool {
    let path = Path::new(INPUT_FILE_PATH);
    return path.exists();
}

/// This obtains the expected type `T` from a discovery details String by running it through function `f` which will
/// attempt to deserialize JSON the String.
pub fn deserialize_discovery_details<T>(discovery_details: &str) -> Result<T, anyhow::Error>
where
    T: serde::de::DeserializeOwned,
{
    let discovery_handler_config: T = serde_json::from_str(discovery_details)?;
    Ok(discovery_handler_config)
}
