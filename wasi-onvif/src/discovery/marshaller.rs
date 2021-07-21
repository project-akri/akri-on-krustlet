use serde_json::Value;

pub fn from_json_to_url_list(json_array: &serde_json::Value) -> Vec<String> {
    json_array
        .as_array()
        .unwrap()
        .iter()
        .map(|value| value.as_str().unwrap().to_string())
        .collect()
}
