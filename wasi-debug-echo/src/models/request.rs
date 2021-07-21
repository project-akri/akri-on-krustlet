use super::device::Device;
use serde::{Deserialize, Serialize};

/// DebugEchoDiscoveryDetails describes the necessary information needed to discover and filter debug echo devices.
/// Specifically, it contains a list (`descriptions`) of fake devices to be discovered.
/// This information is expected to be serialized in the discovery details map sent during Discover requests.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DebugEchoDiscoveryDetails {
    pub descriptions: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DebugEchoResult {
    pub devices: Vec<Device>,
}
