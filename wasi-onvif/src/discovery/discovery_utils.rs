use bytes::Bytes;
use futures_util::stream::TryStreamExt;
use http;
use log::trace;

use std::io::Error;
use std::io::ErrorKind;
use sxd_document::{parser, Package};
use sxd_xpath::Value;
use wasi_experimental_http;

pub const ONVIF_DEVICE_SERVICE_URL_LABEL_ID: &str = "ONVIF_DEVICE_SERVICE_URL";
pub const ONVIF_DEVICE_IP_ADDRESS_LABEL_ID: &str = "ONVIF_DEVICE_IP_ADDRESS";
pub const ONVIF_DEVICE_MAC_ADDRESS_LABEL_ID: &str = "ONVIF_DEVICE_MAC_ADDRESS";
pub const MEDIA_WSDL: &str = "http://www.onvif.org/ver10/media/wsdl";
pub const DEVICE_WSDL: &str = "http://www.onvif.org/ver10/device/wsdl";

/// OnvifQuery can access ONVIF properties given an ONVIF camera's device service url.
///
/// An implementation of an onvif query can retrieve the camera's ip/mac address, scopes, profiles and streaming uri.
pub trait OnvifQuery {
    fn get_device_ip_and_mac_address(
        &self,
        service_url: &str,
    ) -> Result<(String, String), anyhow::Error>;
    fn get_device_scopes(&self, url: &str) -> Result<Vec<String>, anyhow::Error>;
    fn get_device_service_uri(&self, url: &str, service: &str) -> Result<String, anyhow::Error>;
    fn get_device_profiles(&self, url: &str) -> Result<Vec<String>, anyhow::Error>;
    fn get_device_profile_streaming_uri(
        &self,
        url: &str,
        profile_token: &str,
    ) -> Result<String, anyhow::Error>;
}

pub struct OnvifQueryImpl {}

impl OnvifQuery for OnvifQueryImpl {
    /// Gets the ip and mac address of a given ONVIF camera
    fn get_device_ip_and_mac_address(
        &self,
        service_url: &str,
    ) -> Result<(String, String), anyhow::Error> {
        let http = HttpRequest {};
        inner_get_device_ip_and_mac_address(service_url, &http)
    }

    /// Gets the list of scopes for a given ONVIF camera
    fn get_device_scopes(&self, url: &str) -> Result<Vec<String>, anyhow::Error> {
        let http = HttpRequest {};
        inner_get_device_scopes(url, &http)
    }

    /// Gets specific service, like media, from a given ONVIF camera
    fn get_device_service_uri(&self, url: &str, service: &str) -> Result<String, anyhow::Error> {
        let http = HttpRequest {};
        inner_get_device_service_uri(url, service, &http)
    }

    /// Gets the list of streaming profiles for a given ONVIF camera
    fn get_device_profiles(&self, url: &str) -> Result<Vec<String>, anyhow::Error> {
        let http = HttpRequest {};
        inner_get_device_profiles(url, &http)
    }

    /// Gets the streaming uri for a given ONVIF camera's profile
    fn get_device_profile_streaming_uri(
        &self,
        url: &str,
        profile_token: &str,
    ) -> Result<String, anyhow::Error> {
        let http = HttpRequest {};
        inner_get_device_profile_streaming_uri(url, profile_token, &http)
    }
}

/// Http can send an HTTP::Post.
///
/// An implementation of http can send an HTTP::Post.
trait Http {
    fn post(&self, url: &str, mime_action: &str, msg: &str) -> Result<Package, anyhow::Error>;
}

struct HttpRequest {}

impl HttpRequest {
    /// This converts an http response body into an sxd_document::Package
    fn handle_request_body(body: &str) -> Result<Package, anyhow::Error> {
        let xml_as_tree = match parser::parse(&body) {
            Ok(xml_as_tree) => xml_as_tree,
            Err(e) => return Err(std::io::Error::new(ErrorKind::InvalidData, e).into()),
        };
        trace!(
            "handle_request_body - response as xmltree: {:?}",
            xml_as_tree
        );
        Ok(xml_as_tree)
    }
}

impl Http for HttpRequest {
    /// This sends an HTTP::Post and converts the response body into an sxd_document::Package
    fn post(&self, url: &str, mime_action: &str, msg: &str) -> Result<Package, anyhow::Error> {
        trace!(
            "post - url:{}, mime_action:{}, msg:{}",
            &url,
            &mime_action,
            &msg
        );

        let full_mime = format!(
            "{}; {}; {};",
            "application/soap+xml", "charset=utf-8", mime_action
        );

        let b = Bytes::from(msg.to_string());
        let request = http::request::Builder::new()
            .method(http::Method::POST)
            .uri(url)
            .header("CONTENT-TYPE", full_mime)
            .body(Some(b))
            .unwrap();

        println!("{:?}", request);

        let mut response = wasi_experimental_http::request(request).expect("cannot make request");
        if response.status_code != 200 {
            return Err(anyhow::format_err!("failure"));
        }
        let response_body_str = std::str::from_utf8(&response.body_read_all().unwrap())
            .unwrap()
            .to_string();
        match HttpRequest::handle_request_body(&response_body_str) {
            Ok(dom) => Ok(dom),
            Err(e) => {
                trace!(
                    "post - failure to handle response: {:?}",
                    &response_body_str
                );
                Err(e)
            }
        }
    }
}

/// Creates a SOAP mime action
fn get_action(wsdl: &str, function: &str) -> String {
    format!("action=\"{}/{}\"", wsdl, function)
}

/// Gets the ip and mac address for a given ONVIF camera
fn inner_get_device_ip_and_mac_address(
    service_url: &str,
    http: &impl Http,
) -> Result<(String, String), anyhow::Error> {
    let network_interfaces_xml = match http.post(
        service_url,
        &get_action(DEVICE_WSDL, "GetNetworkInterfaces"),
        &GET_NETWORK_INTERFACES_TEMPLATE.to_string(),
    ) {
        Ok(xml) => xml,
        Err(e) => {
            return Err(anyhow::format_err!(
                "failed to get network interfaces from device: {:?}",
                e
            ))
        }
    };
    let network_interfaces_doc = network_interfaces_xml.as_document();
    let ip_address = match sxd_xpath::evaluate_xpath(
            &network_interfaces_doc,
            "//*[local-name()='GetNetworkInterfacesResponse']/*[local-name()='NetworkInterfaces']/*[local-name()='IPv4']/*[local-name()='Config']/*/*[local-name()='Address']/text()"
        ) {
            Ok(Value::String(ip)) => ip,
            Ok(Value::Nodeset(ns)) => match ns.into_iter().map(|x| x.string_value()).collect::<Vec<String>>().first() {
                Some(first) => first.to_string(),
                None => return Err(anyhow::format_err!("Failed to get ONVIF ip address: none specified in response"))
            },
            Ok(Value::Boolean(_)) |
            Ok(Value::Number(_)) => return Err(anyhow::format_err!("Failed to get ONVIF ip address: unexpected type")),
            Err(e) => return Err(anyhow::format_err!("Failed to get ONVIF ip address: {}", e))
        };
    trace!(
        "inner_get_device_ip_and_mac_address - network interfaces (ip address): {:?}",
        ip_address
    );
    let mac_address = match sxd_xpath::evaluate_xpath(
            &network_interfaces_doc,
            "//*[local-name()='GetNetworkInterfacesResponse']/*[local-name()='NetworkInterfaces']/*[local-name()='Info']/*[local-name()='HwAddress']/text()"
        ) {
            Ok(Value::String(mac)) => mac,
            Ok(Value::Nodeset(ns)) => match ns.iter().map(|x| x.string_value()).collect::<Vec<String>>().first() {
                Some(first) => first.to_string(),
                None => return Err(anyhow::format_err!("Failed to get ONVIF mac address: none specified in response"))
            },
            Ok(Value::Boolean(_)) |
            Ok(Value::Number(_)) => return Err(anyhow::format_err!("Failed to get ONVIF mac address: unexpected type")),
            Err(e) => return Err(anyhow::format_err!("Failed to get ONVIF mac address: {}", e))
        };
    trace!(
        "inner_get_device_ip_and_mac_address - network interfaces (mac address): {:?}",
        mac_address
    );
    Ok((ip_address, mac_address))
}

/// Gets the list of scopes for a given ONVIF camera
fn inner_get_device_scopes(url: &str, http: &impl Http) -> Result<Vec<String>, anyhow::Error> {
    let scopes_xml = match http.post(
        &url,
        &get_action(DEVICE_WSDL, "GetScopes"),
        &GET_SCOPES_TEMPLATE.to_string(),
    ) {
        Ok(xml) => xml,
        Err(e) => {
            return Err(anyhow::format_err!(
                "failed to get scopes from device: {:?}",
                e
            ))
        }
    };
    let scopes_doc = scopes_xml.as_document();
    let scopes_query = sxd_xpath::evaluate_xpath(
        &scopes_doc,
        "//*[local-name()='GetScopesResponse']/*[local-name()='Scopes']/*[local-name()='ScopeItem']/text()"
    );
    let scopes = match scopes_query {
        Ok(Value::Nodeset(scope_items)) => scope_items
            .iter()
            .map(|scope_item| scope_item.string_value())
            .collect::<Vec<String>>(),
        Ok(Value::Boolean(_)) | Ok(Value::Number(_)) | Ok(Value::String(_)) => {
            return Err(anyhow::format_err!(
                "Failed to get ONVIF scopes: unexpected type"
            ))
        }
        Err(e) => return Err(anyhow::format_err!("Failed to get ONVIF scopes: {}", e)),
    };
    trace!("inner_get_device_scopes - scopes: {:?}", scopes);
    Ok(scopes)
}

/// SOAP request body for getting the network interfaces for an ONVIF camera
const GET_NETWORK_INTERFACES_TEMPLATE: &str = r#"<soap:Envelope xmlns:soap="http://www.w3.org/2003/05/soap-envelope" xmlns:wsdl="http://www.onvif.org/ver10/device/wsdl">
    <soap:Header/>
        <soap:Body>
            <wsdl:GetNetworkInterfaces/>
        </soap:Body>
    </soap:Envelope>"#;

/// SOAP request body for getting scopes for an ONVIF camera
const GET_SCOPES_TEMPLATE: &str = r#"<soap:Envelope xmlns:soap="http://www.w3.org/2003/05/soap-envelope" xmlns:wsdl="http://www.onvif.org/ver10/device/wsdl">
    <soap:Header/>
        <soap:Body>
            <wsdl:GetScopes/>
        </soap:Body>
    </soap:Envelope>"#;

/// Gets a specific service (like media) uri from an ONVIF camera
fn inner_get_device_service_uri(
    url: &str,
    service: &str,
    http: &impl Http,
) -> Result<String, anyhow::Error> {
    let services_xml = match http.post(
        &url,
        &get_action(DEVICE_WSDL, "GetServices"),
        &GET_SERVICES_TEMPLATE.to_string(),
    ) {
        Ok(xml) => xml,
        Err(e) => {
            return Err(anyhow::format_err!(
                "failed to get services from device: {:?}",
                e
            ))
        }
    };
    let services_doc = services_xml.as_document();
    let service_xpath_query = format!(
        "//*[local-name()='GetServicesResponse']/*[local-name()='Service' and *[local-name()='Namespace']/text() ='{}']/*[local-name()='XAddr']/text()",
        service
    );
    let requested_device_service_uri =
        match sxd_xpath::evaluate_xpath(&services_doc, service_xpath_query.as_str()) {
            Ok(uri) => uri.string(),
            Err(e) => {
                return Err(anyhow::format_err!(
                    "failed to get servuce uri from resoinse: {:?}",
                    e
                ))
            }
        };
    trace!(
        "inner_get_device_service_uri - service ({}) uris: {:?}",
        service,
        requested_device_service_uri
    );
    Ok(requested_device_service_uri)
}

/// SOAP request body for getting the supported services' uris for an ONVIF camera
const GET_SERVICES_TEMPLATE: &str = r#"<soap:Envelope xmlns:soap="http://www.w3.org/2003/05/soap-envelope" xmlns:wsdl="http://www.onvif.org/ver10/device/wsdl">
    <soap:Header/>
        <soap:Body>
            <wsdl:GetServices />
        </soap:Body>
    </soap:Envelope>"#;

/// Gets list of media profiles for a given ONVIF camera
fn inner_get_device_profiles(url: &str, http: &impl Http) -> Result<Vec<String>, anyhow::Error> {
    let action = get_action(MEDIA_WSDL, "GetProfiles");
    let message = GET_PROFILES_TEMPLATE.to_string();
    let profiles_xml = match http.post(&url, &action, &message) {
        Ok(xml) => xml,
        Err(e) => {
            return Err(anyhow::format_err!(
                "failed to get profiles from device: {:?}",
                e
            ))
        }
    };
    let profiles_doc = profiles_xml.as_document();
    let profiles_query = sxd_xpath::evaluate_xpath(
        &profiles_doc,
        "//*[local-name()='GetProfilesResponse']/*[local-name()='Profiles']/@token",
    );
    let profiles = match profiles_query {
        Ok(Value::Nodeset(profiles_items)) => profiles_items
            .iter()
            .map(|profile_item| profile_item.string_value())
            .collect::<Vec<String>>(),
        Ok(Value::Boolean(_)) | Ok(Value::Number(_)) | Ok(Value::String(_)) => {
            return Err(anyhow::format_err!(
                "Failed to get ONVIF profiles: unexpected type"
            ))
        }
        Err(e) => return Err(anyhow::format_err!("Failed to get ONVIF profiles: {}", e)),
    };
    trace!("inner_get_device_scopes - profiles: {:?}", profiles);
    Ok(profiles)
}

/// Gets the streaming uri for a given profile for an ONVIF camera
fn inner_get_device_profile_streaming_uri(
    url: &str,
    profile_token: &str,
    http: &impl Http,
) -> Result<String, anyhow::Error> {
    let stream_soap = get_stream_uri_message(&profile_token);
    let stream_uri_xml =
        match http.post(&url, &get_action(MEDIA_WSDL, "GetStreamUri"), &stream_soap) {
            Ok(xml) => xml,
            Err(e) => {
                return Err(anyhow::format_err!(
                    "failed to get streaming uri from device: {:?}",
                    e
                ))
            }
        };
    let stream_uri_doc = stream_uri_xml.as_document();
    let stream_uri = match sxd_xpath::evaluate_xpath(
        &stream_uri_doc,
        "//*[local-name()='GetStreamUriResponse']/*[local-name()='MediaUri']/*[local-name()='Uri']/text()"
        ) {
            Ok(stream) => stream.string(),
            Err(e) => {
                return Err(anyhow::format_err!(
                    "failed to get servuce uri from resoinse: {:?}",
                    e
                ))
            }
        };
    Ok(stream_uri)
}

/// Gets SOAP request body for getting the streaming uri for a specific profile for an ONVIF camera
fn get_stream_uri_message(profile: &str) -> String {
    format!(
        r#"<soap:Envelope xmlns:soap="http://www.w3.org/2003/05/soap-envelope" xmlns:wsdl="http://www.onvif.org/ver10/media/wsdl" xmlns:sch="http://www.onvif.org/ver10/schema">
            <soap:Header/>
            <soap:Body>
                <wsdl:GetStreamUri>
                <wsdl:StreamSetup>
                    <sch:Stream>RTP-Unicast</sch:Stream>
                    <sch:Transport>
                        <sch:Protocol>RTSP</sch:Protocol>
                    </sch:Transport>
                </wsdl:StreamSetup>
                <wsdl:ProfileToken>{}</wsdl:ProfileToken>
                </wsdl:GetStreamUri>
            </soap:Body>
        </soap:Envelope>;"#,
        profile
    )
}

/// SOAP request body for getting the media profiles for an ONVIF camera
const GET_PROFILES_TEMPLATE: &str = r#"<soap:Envelope xmlns:soap="http://www.w3.org/2003/05/soap-envelope" xmlns:wsdl="http://www.onvif.org/ver10/media/wsdl">
    <soap:Header/>
        <soap:Body>
            <wsdl:GetProfiles/>
        </soap:Body>
    </soap:Envelope>"#;

//  const GET_DEVICE_INFORMATION_TEMPLATE: &str = r#"<soap:Envelope xmlns:soap="http://www.w3.org/2003/05/soap-envelope" xmlns:wsdl="http://www.onvif.org/ver10/device/wsdl">
//     <soap:Header/>
//         <soap:Body>
//             <wsdl:GetDeviceInformation/>
//         </soap:Body>
//     </soap:Envelope>"#;

//  const GET_HOSTNAME_TEMPLATE: &str = r#"<soap:Envelope xmlns:soap="http://www.w3.org/2003/05/soap-envelope" xmlns:wsdl="http://www.onvif.org/ver10/device/wsdl">
//     <soap:Header/>
//         <soap:Body>
//             <wsdl:GetHostname/>
//         </soap:Body>
//     </soap:Envelope>"#;
