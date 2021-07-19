use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, ::prost::Message)]
pub struct Device {
    /// Identifier for this device
    #[prost(string, tag = "1")]
    pub id: std::string::String,
    /// Properties that identify the device. These are stored in the device's instance
    /// and set as environment variables in the device's broker Pods. May be information
    /// about where to find the device such as an RTSP URL or a device node (e.g. `/dev/video1`)
    #[prost(map = "string, string", tag = "2")]
    pub properties: ::std::collections::HashMap<std::string::String, std::string::String>,
    /// Optionally specify mounts for Pods that request this device as a resource
    #[prost(message, repeated, tag = "3")]
    pub mounts: ::std::vec::Vec<Mount>,
    /// Optionally specify device information to be mounted for Pods that request this device as a resource
    #[prost(message, repeated, tag = "4")]
    pub device_specs: ::std::vec::Vec<DeviceSpec>,
}
/// From Device Plugin  API
/// Mount specifies a host volume to mount into a container.
/// where device library or tools are installed on host and container
#[derive(Serialize, Deserialize, Clone, PartialEq, ::prost::Message)]
pub struct Mount {
    /// Path of the mount within the container.
    #[prost(string, tag = "1")]
    pub container_path: std::string::String,
    /// Path of the mount on the host.
    #[prost(string, tag = "2")]
    pub host_path: std::string::String,
    /// If set, the mount is read-only.
    #[prost(bool, tag = "3")]
    pub read_only: bool,
}
/// From Device Plugin API
/// DeviceSpec specifies a host device to mount into a container.
#[derive(Serialize, Deserialize, Clone, PartialEq, ::prost::Message)]
pub struct DeviceSpec {
    /// Path of the device within the container.
    #[prost(string, tag = "1")]
    pub container_path: std::string::String,
    /// Path of the device on the host.
    #[prost(string, tag = "2")]
    pub host_path: std::string::String,
    /// Cgroups permissions of the device, candidates are one or more of
    /// * r - allows container to read from the specified device.
    /// * w - allows container to write to the specified device.
    /// * m - allows container to create device files that do not yet exist.
    #[prost(string, tag = "3")]
    pub permissions: std::string::String,
}