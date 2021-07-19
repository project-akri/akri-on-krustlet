use serde::{Serialize, Deserialize};
use super::device::Device;

/// This defines the ONVIF data stored in the Configuration
/// CRD
///
/// The ONVIF discovery handler is structured to store a filter list for
/// ip addresses, mac addresses, and ONVIF scopes.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct OnvifDiscoveryDetails {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ip_addresses: Option<FilterList>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mac_addresses: Option<FilterList>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scopes: Option<FilterList>,
    #[serde(default = "default_discovery_timeout_seconds")]
    pub discovery_timeout_seconds: i32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct OnvifResult {
    pub devices: Vec<Device>,
}

fn default_discovery_timeout_seconds() -> i32 {
    1
}

/// This defines the types of supported filters
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum FilterType {
    /// If the filter type is Exclude, any items NOT found in the
    /// list are accepted
    Exclude,
    /// If the filter type is Include, only items found in the
    /// list are accepted
    Include,
}

/// The default filter type is `Include`
fn default_action() -> FilterType {
    FilterType::Include
}

/// This defines a filter list.
///
/// The items list can either define the only acceptable
/// items (Include) or can define the only unacceptable items
/// (Exclude)
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FilterList {
    /// This defines a list of items that will be evaluated as part
    /// of the filtering process
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub items: Vec<String>,
    /// This defines what the evaluation of items will be.  The default
    /// is `Include`
    #[serde(default = "default_action")]
    pub action: FilterType,
}
