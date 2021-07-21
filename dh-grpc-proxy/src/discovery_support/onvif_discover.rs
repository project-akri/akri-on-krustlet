pub mod to_serialize {
    use super::common::*;
    #[derive(Default, PartialEq, Debug, YaSerialize)]
    #[yaserde(prefix = "s", namespace = "s: http://www.w3.org/2003/05/soap-envelope")]
    pub struct Envelope {
        #[yaserde(prefix = "s", rename = "Header")]
        pub header: Header,

        #[yaserde(prefix = "s", rename = "Body")]
        pub body: Body,
    }

    #[derive(Default, PartialEq, Debug, YaSerialize)]
    #[yaserde(
        prefix = "s",
        namespace = "s: http://www.w3.org/2003/05/soap-envelope",
        namespace = "d: http://schemas.xmlsoap.org/ws/2005/04/discovery"
    )]
    pub struct Body {
        #[yaserde(prefix = "d", rename = "Probe")]
        pub probe: Probe,
    }

    #[derive(Default, PartialEq, Debug, YaSerialize)]
    #[yaserde(
        prefix = "s",
        namespace = "s: http://www.w3.org/2003/05/soap-envelope",
        namespace = "w: http://schemas.xmlsoap.org/ws/2004/08/addressing"
    )]
    pub struct Header {
        #[yaserde(prefix = "w", rename = "MessageID")]
        pub message_id: String,

        #[yaserde(prefix = "w", rename = "To")]
        pub reply_to: String,

        #[yaserde(prefix = "w", rename = "Action")]
        pub action: String,
    }
}

pub mod to_deserialize {
    use super::common::*;

    #[derive(Default, PartialEq, Debug, YaDeserialize)]
    #[yaserde(prefix = "s", namespace = "s: http://www.w3.org/2003/05/soap-envelope")]
    pub struct Envelope {
        #[yaserde(prefix = "s", rename = "Header")]
        pub header: Header,

        #[yaserde(prefix = "s", rename = "Body")]
        pub body: Body,
    }

    #[derive(Default, PartialEq, Debug, YaDeserialize)]
    #[yaserde(
        prefix = "s",
        namespace = "s: http://www.w3.org/2003/05/soap-envelope",
        namespace = "d: http://schemas.xmlsoap.org/ws/2005/04/discovery"
    )]
    pub struct Body {
        #[yaserde(prefix = "d", rename = "ProbeMatches")]
        pub probe_matches: ProbeMatches,
    }

    #[derive(Default, PartialEq, Debug, YaDeserialize)]
    #[yaserde(
        prefix = "s",
        namespace = "s: http://www.w3.org/2003/05/soap-envelope",
        namespace = "w: http://schemas.xmlsoap.org/ws/2004/08/addressing"
    )]
    pub struct Header {
        #[yaserde(prefix = "w", rename = "RelatesTo")]
        pub relates_to: String,
    }
}

#[allow(dead_code)]
pub mod probe_types {
    pub const DEVICE_NAMESPACE_PREFIX: &str = "devwsdl";
    pub const NETWORK_VIDEO_TRANSMITTER_NAMESPACE_PREFIX: &str = "netwsdl";
    pub const DEVICE_NAMESPACE_DESCRIPTOR: &str = "devwsdl: http://www.onvif.org/ver10/device/wsdl";
    pub const NETWORK_VIDEO_TRANSMITTER_NAMESPACE_DESCRIPTOR: &str =
        "netwsdl: http://www.onvif.org/ver10/network/wsdl";
    pub const DEVICE: &str = "devwsdl:Device";
    pub const NETWORK_VIDEO_TRANSMITTER: &str = "netwsdl:NetworkVideoTransmitter";
}

pub mod common {
    #[derive(Default, PartialEq, Debug, YaDeserialize, YaSerialize)]
    #[yaserde(
        prefix = "d",
        namespace = "d: http://schemas.xmlsoap.org/ws/2005/04/discovery",
        namespace = probe_typews::NETWORK_VIDEO_TRANSMITTER_NAMESPACE_DESCRIPTOR,
        namespace = probe_typews::DEVICE_NAMESPACE_DESCRIPTOR
    )]
    pub struct Probe {
        #[yaserde(prefix = "d", rename = "Types")]
        pub probe_types: Vec<String>,
    }

    #[derive(Default, PartialEq, Debug, YaDeserialize, YaSerialize)]
    #[yaserde(
        prefix = "d",
        namespace = "d: http://schemas.xmlsoap.org/ws/2005/04/discovery",
        namespace = "wsa: http://schemas.xmlsoap.org/ws/2004/08/addressing"
    )]
    pub struct ProbeMatch {
        #[yaserde(prefix = "d", rename = "XAddrs")]
        pub xaddrs: String,
        #[yaserde(prefix = "wsa", rename = "EndpointReference")]
        pub endpoint_reference: String,
        #[yaserde(prefix = "d", rename = "Types")]
        pub probe_types: Vec<String>,
        #[yaserde(prefix = "d", rename = "Scopes")]
        pub scopes: Vec<String>,
        #[yaserde(prefix = "d", rename = "MetadataVersion")]
        pub metadata_version: String,
    }

    #[derive(Default, PartialEq, Debug, YaDeserialize, YaSerialize)]
    #[yaserde(
        prefix = "d",
        namespace = "d: http://schemas.xmlsoap.org/ws/2005/04/discovery"
    )]
    pub struct ProbeMatches {
        #[yaserde(prefix = "d", rename = "ProbeMatch")]
        pub probe_match: Vec<ProbeMatch>,
    }
}

pub mod util {
    use super::{common, probe_types, to_deserialize, to_serialize};
    use log::{error, info, trace};
    use std::{
        net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket},
        sync::{Arc, Mutex},
    };
    use tokio::{
        io::ErrorKind,
        sync::{mpsc, mpsc::error::TryRecvError},
        time,
        time::Duration,
    };

    fn create_onvif_discovery_message(uuid_string: &str) -> String {
        let probe_types: Vec<String> = vec![probe_types::NETWORK_VIDEO_TRANSMITTER.into()];
        let envelope = to_serialize::Envelope {
            header: to_serialize::Header {
                message_id: uuid_string.into(),
                action: "http://schemas.xmlsoap.org/ws/2005/04/discovery/Probe".into(),
                reply_to: "urn:schemas-xmlsoap-org:ws:2005:04:discovery".into(),
            },
            body: to_serialize::Body {
                probe: common::Probe { probe_types },
            },
        };
        let envelope_as_string = yaserde::ser::to_string(&envelope).unwrap();
        trace!(
            "create_onvif_discovery_message - discovery message: {:?}",
            &envelope_as_string
        );
        envelope_as_string
    }

    fn get_device_uris_from_discovery_response(discovery_response: &str) -> Vec<String> {
        let response_envelope =
            yaserde::de::from_str::<to_deserialize::Envelope>(&discovery_response);
        // The response envelope follows this format:
        //   <Envelope><Body><ProbeMatches><ProbeMatch><XAddrs>
        //       https://10.0.0.1:5357/svc
        //       https://10.0.0.2:5357/svc
        //       https://10.0.0.3:5357/svc
        //   </ProbeMatch></ProbeMatches></XAddrs></Body></Envelope>
        response_envelope
            .unwrap()
            .body
            .probe_matches
            .probe_match
            .iter()
            .flat_map(|probe_match| probe_match.xaddrs.split_whitespace())
            .map(|addr| addr.to_string())
            .collect::<Vec<String>>()
    }

    pub async fn simple_onvif_discover(timeout: Duration) -> Result<Vec<String>, anyhow::Error> {
        let (mut discovery_timeout_tx, mut discovery_timeout_rx) = mpsc::channel(2);
        let (mut discovery_cancel_tx, mut discovery_cancel_rx) = mpsc::channel(2);
        let shared_devices = Arc::new(Mutex::new(Vec::new()));

        let uuid_str = format!("uuid:{}", uuid::Uuid::new_v4());
        trace!("simple_onvif_discover - for {}", &uuid_str);

        let thread_devices = shared_devices.clone();
        tokio::spawn(async move {
            trace!(
                "simple_onvif_discover - spawned thread enter for {}",
                &uuid_str
            );

            const LOCAL_IPV4_ADDR: Ipv4Addr = Ipv4Addr::UNSPECIFIED;
            const LOCAL_PORT: u16 = 0;
            let local_socket_addr = SocketAddr::new(IpAddr::V4(LOCAL_IPV4_ADDR), LOCAL_PORT);

            // WS-Discovery multicast ip and port selected from available standard
            // options.  See https://en.wikipedia.org/wiki/WS-Discovery
            const MULTI_IPV4_ADDR: Ipv4Addr = Ipv4Addr::new(239, 255, 255, 250);
            const MULTI_PORT: u16 = 3702;
            let multi_socket_addr = SocketAddr::new(IpAddr::V4(MULTI_IPV4_ADDR), MULTI_PORT);

            trace!(
                "simple_onvif_discover - binding to: {:?}",
                local_socket_addr
            );
            let socket = UdpSocket::bind(local_socket_addr).unwrap();
            socket
                .set_write_timeout(Some(Duration::from_millis(200)))
                .unwrap();
            socket
                .set_read_timeout(Some(Duration::from_millis(200)))
                .unwrap();
            trace!(
                "simple_onvif_discover - joining multicast: {:?} {:?}",
                &MULTI_IPV4_ADDR,
                &LOCAL_IPV4_ADDR
            );
            socket
                .join_multicast_v4(&MULTI_IPV4_ADDR, &LOCAL_IPV4_ADDR)
                .unwrap();

            let envelope_as_string = create_onvif_discovery_message(&uuid_str);
            match socket.send_to(&envelope_as_string.as_bytes(), multi_socket_addr) {
                Ok(_) => {
                    loop {
                        let mut buf = vec![0; 16 * 1024];
                        match socket.recv_from(&mut buf) {
                            Ok((len, _)) => {
                                let broadcast_response_as_string =
                                    String::from_utf8_lossy(&buf[..len]).to_string();
                                trace!(
                                    "simple_onvif_discover - response: {:?}",
                                    broadcast_response_as_string
                                );

                                get_device_uris_from_discovery_response(
                                    &broadcast_response_as_string,
                                )
                                .iter()
                                .for_each(|device_uri| {
                                    trace!(
                                        "simple_onvif_discover - device_uri parsed from response: {:?}",
                                        device_uri
                                    );
                                    thread_devices.lock().unwrap().push(device_uri.to_string());
                                    trace!(
                                        "simple_onvif_discover - thread_devices: {:?}",
                                        thread_devices.lock().unwrap()
                                    );
                                });
                            }
                            Err(e) => match e.kind() {
                                ErrorKind::WouldBlock | ErrorKind::TimedOut => {
                                    match discovery_cancel_rx.try_recv() {
                                        Err(TryRecvError::Closed) | Ok(_) => {
                                            trace!("simple_onvif_discover - recv_from error ... timeout signalled/disconnected (stop collecting responses): {:?}", e);
                                            break;
                                        }
                                        Err(TryRecvError::Empty) => {
                                            trace!("simple_onvif_discover - recv_from error ... no timeout (continue collecting responses): {:?}", e);
                                            // continue looping
                                        }
                                    }
                                }
                                e => {
                                    error!("simple_onvif_discover - recv_from error: {:?}", e);
                                    Err(e).unwrap()
                                }
                            },
                        }
                    }
                }
                Err(e) => match e.kind() {
                    ErrorKind::WouldBlock | ErrorKind::TimedOut => {
                        trace!("simple_onvif_discover - send_to timeout: {:?}", e);
                        return;
                    }
                    e => {
                        error!("simple_onvif_discover - send_to error: {:?}", e);
                        Err(e).unwrap()
                    }
                },
            }

            let _best_effort_send = discovery_timeout_tx.send(()).await;
            trace!("simple_onvif_discover - spawned thread exit");
        });

        // Wait for timeout for discovery thread
        let discovery_timeout_rx_result = time::timeout(
            Duration::from_secs(timeout.as_secs()),
            discovery_timeout_rx.recv(),
        )
        .await;
        trace!(
            "simple_onvif_discover - spawned thread finished or timeout: {:?}",
            discovery_timeout_rx_result
        );
        // Send cancel message to thread to ensure it doesn't hang around
        let _best_effort_cancel = discovery_cancel_tx.send(()).await;

        let result_devices = shared_devices.lock().unwrap().clone();
        info!("simple_onvif_discover - devices: {:?}", result_devices);
        Ok(result_devices)
    }
}
