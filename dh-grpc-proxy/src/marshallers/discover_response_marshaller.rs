use akri_discovery_utils::discovery::v0::{Device, DiscoverResponse};
use serde_json::Serializer;
use serde_json::Value;

pub fn from_json_to_discover_response(json_str: &str) -> DiscoverResponse {
    let json: Value = serde_json::from_str(json_str).unwrap();

    DiscoverResponse {
        devices: json["devices"]
            .as_array()
            .unwrap()
            .iter()
            .map(|device_json| from_json_to_device(device_json))
            .collect(),
    }
}

pub fn from_json_to_device(json_device: &serde_json::Value) -> Device {
    Device {
        id: json_device["id"].to_string(),
        properties: json_device["properties"]
            .as_object()
            .unwrap()
            .iter()
            .map(|entry| (entry.0.clone(), entry.1.clone().to_string()))
            .collect(),
        device_specs: Vec::new(),
        mounts: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::from_json_to_discover_response;
    use akri_discovery_utils::discovery::v0::Device;
    use akri_discovery_utils::discovery::v0::DiscoverResponse;

    #[test]
    fn test_marshall_discover_request() {
        let discover_response_json = r#"
        {
            "devices": [
                {
                    "id":"foo0",
                    "properties":{"DEBUG_ECHO_DESCRIPTION":"foo0"},
                    "mounts":[]
                    ,"device_specs":[]
                },
                {
                    "id":"foo1",
                    "properties":{"DEBUG_ECHO_DESCRIPTION":"foo1"},
                    "mounts":[],
                    "device_specs":[]
                },
                {
                    "id":"foo2",
                    "properties":{"DEBUG_ECHO_DESCRIPTION":"foo2"},
                    "mounts":[],
                    "device_specs":[]
                }
            ]
        }"#;

        let discover_response_obj: DiscoverResponse =
            from_json_to_discover_response(&discover_response_json);
        println!("{:?}", discover_response_obj);
        assert_eq!(discover_response_obj.devices.len(), 3);
    }
}
