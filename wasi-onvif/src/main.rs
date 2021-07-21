mod discovery;
mod models;

use discovery::discovery_utils::{
    OnvifQuery, OnvifQueryImpl, ONVIF_DEVICE_IP_ADDRESS_LABEL_ID,
    ONVIF_DEVICE_MAC_ADDRESS_LABEL_ID, ONVIF_DEVICE_SERVICE_URL_LABEL_ID,
};

use discovery::marshaller;
use log::{error, info, trace};
use models::device::Device;
use models::request::{FilterList, FilterType, OnvifDiscoveryDetails, OnvifResult};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::thread;
use std::time::Duration;

#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate yaserde_derive;

// Input and output files dir.
pub const OUTPUT_FILE_PATH: &str = "/tmp/wonvif-dir/out.out";
pub const INPUT_FILE_PATH: &str = "/tmp/wonvif-dir/in.in";
pub const DEBUG_FILE_PATH: &str = "/tmp/wonvif-dir/debug.txt";
pub const URLS_FILE_PATH: &str = "/tmp/wonvif-dir/onvif-urls.txt";

pub const DISCOVERY_INTERVAL_SECS: u64 = 5;

fn main() {
    // Input variables
    let mut input: OnvifDiscoveryDetails;
    println!("Wasi Onvif Discovery Handler running! :)");
    // write_debug_file(DEBUG_FILE_PATH, "Hello World!");

    let mut cameras: &Vec<Device> = &Vec::new();

    loop {
        // thread::sleep(Duration::from_secs(DISCOVERY_INTERVAL_SECS));

        if !has_input() {
            println!("Input not specified yet!");
            // continue;
        }
        let onvif_query = OnvifQueryImpl {};
        input = read_input_file();

        trace!("discover - filters:{:?}", &input);
        let discovered_onvif_cameras = read_url_file();

        trace!("discover - got back with:{:?}", &discovered_onvif_cameras);

        // apply_filters never returns an error -- safe to unwrap
        let filtered_onvif_cameras =
            &apply_filters(&input, discovered_onvif_cameras, &onvif_query).unwrap();

        trace!("discover - filtered:{:?}", filtered_onvif_cameras);

        let mut changed_camera_list = false;
        let mut matching_camera_count = 0;
        filtered_onvif_cameras.iter().for_each(|camera| {
            if !cameras.contains(&camera.clone()) {
                changed_camera_list = true;
            } else {
                matching_camera_count += 1;
            }
        });
        if changed_camera_list || matching_camera_count != cameras.len() {
            trace!("discover - sending updated device list");
            // cameras = &filtered_onvif_cameras;
            write_output_file(filtered_onvif_cameras);
        }
    }
}

fn apply_filters(
    discovery_handler_config: &OnvifDiscoveryDetails,
    device_service_uris: Vec<String>,
    onvif_query: &impl OnvifQuery,
) -> Result<Vec<Device>, anyhow::Error> {
    let mut result = Vec::new();
    for device_service_url in device_service_uris.iter() {
        trace!("apply_filters - device service url {}", &device_service_url);
        let (ip_address, mac_address) =
            match onvif_query.get_device_ip_and_mac_address(&device_service_url) {
                Ok(ip_and_mac) => ip_and_mac,
                Err(e) => {
                    error!("apply_filters - error getting ip and mac address: {}", e);
                    continue;
                }
            };

        // Evaluate camera ip address against ip filter if provided
        let ip_address_as_vec = vec![ip_address.clone()];
        if execute_filter(
            discovery_handler_config.ip_addresses.as_ref(),
            &ip_address_as_vec,
        ) {
            continue;
        }

        // Evaluate camera mac address against mac filter if provided
        let mac_address_as_vec = vec![mac_address.clone()];
        if execute_filter(
            discovery_handler_config.mac_addresses.as_ref(),
            &mac_address_as_vec,
        ) {
            continue;
        }

        let ip_and_mac_joined = format!("{}-{}", &ip_address, &mac_address);

        // Evaluate camera scopes against scopes filter if provided
        let device_scopes = match onvif_query.get_device_scopes(&device_service_url) {
            Ok(scopes) => scopes,
            Err(e) => {
                error!("apply_filters - error getting scopes: {}", e);
                continue;
            }
        };
        if execute_filter(discovery_handler_config.scopes.as_ref(), &device_scopes) {
            continue;
        }

        let mut properties = HashMap::new();
        properties.insert(
            ONVIF_DEVICE_SERVICE_URL_LABEL_ID.to_string(),
            device_service_url.to_string(),
        );
        properties.insert(ONVIF_DEVICE_IP_ADDRESS_LABEL_ID.into(), ip_address);
        properties.insert(ONVIF_DEVICE_MAC_ADDRESS_LABEL_ID.into(), mac_address);

        trace!(
            "apply_filters - returns DiscoveryResult ip/mac: {:?}, props: {:?}",
            &ip_and_mac_joined,
            &properties
        );
        result.push(Device {
            id: ip_and_mac_joined,
            properties,
            mounts: Vec::default(),
            device_specs: Vec::default(),
        })
    }
    Ok(result)
}

fn execute_filter(filter_list: Option<&FilterList>, filter_against: &[String]) -> bool {
    if filter_list.is_none() {
        return false;
    }
    let filter_action = filter_list.as_ref().unwrap().action.clone();
    let filter_count = filter_list
        .unwrap()
        .items
        .iter()
        .filter(|pattern| {
            filter_against
                .iter()
                .filter(|filter_against_item| filter_against_item.contains(*pattern))
                .count()
                > 0
        })
        .count();

    if FilterType::Include == filter_action {
        filter_count == 0
    } else {
        filter_count != 0
    }
}

// This reads the input file and serialize it to the proper struct format.
pub fn read_input_file() -> OnvifDiscoveryDetails {
    let path = Path::new(INPUT_FILE_PATH);
    let display = path.display();

    // let mut contents = fs::read_to_string(path).expect(format!("could not read {}", display).as_str());

    let mut contents = r#"{
        "ipAddresses":  {
            "action": "Exclude",
            "items": []
        },
        "macAddresses": {
            "action": "Exclude",
            "items": []
        },
        "scopes": {
            "action": "Exclude",
            "items": []
        },
        "discoveryTimeoutSeconds": 1
    }"#.to_string();

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

// This reads the url file and serialize it to the proper struct format.
pub fn read_url_file() -> Vec<String> {
    let path = Path::new(URLS_FILE_PATH);
    let display = path.display();

    // let contents = fs::read_to_string(path).expect(format!("could not read {}", display).as_str());
    // let json: Value = serde_json::from_str(&contents).unwrap();

    let mut result = Vec::new();
    result.push("http://127.0.0.1:1000/onvif/device_service".to_string());

    return result;
    // return marshaller::from_json_to_url_list(&json);
}

// This received the device list and output it in the proper JSON format to the
// output file.
pub fn write_output_file(_devices: &Vec<Device>) {
    let path = Path::new(OUTPUT_FILE_PATH);
    let obj_devices = _devices.clone();

    // Write output values on DebugEchoResult
    let output_obj: OnvifResult = OnvifResult { devices: obj_devices };

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
