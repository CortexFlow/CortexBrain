/* Contains all the functions used to communicate between services */
use base64::{Engine as _, engine::general_purpose::STANDARD};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::net::UdpSocket;
use tokio::{io::AsyncWriteExt, net::TcpStream};
use tracing::{error, info, warn};

/*
Extract the service name and the payload from this format:

               <service_name>.<namespace>:<message>

Messages structure:
    payload
    direction
    service

Message Type:
    Incoming
    Outcoming
    Unknown

*/
/*
    Messagging logic:
        A sends an "Incoming" message to B.
        B receives the message and processes it:
            - If the service is valid, B tries to get a response from service_discovery.
                - If it finds a response, B sends the "Outcoming" message with the correct payload.
            - If it does not find a response, it logs an error.

        A receives an "Outcoming" message with the service response payload.
            - If B receives an "Outcoming" message, it responds with {"status": "received"}.

*/

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum MexType {
    Incoming,
    Outcoming,
    Unknown,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    payload: String, //TODO: consider using type Option<String> so the payload cannot be present and the functions does not return errors
    service: String,
    direction: MexType,
}

pub fn extract_service_name_and_payload(
    msg_encrypted: &[u8],
) -> Option<(MexType, String, Vec<u8>)> {
    // Convert the bytes in a UTF-8 String
    let message = match std::str::from_utf8(msg_encrypted) {
        Ok(msg) => {
            info!("{:?}", msg);
            msg
        }
        Err(e) => {
            error!("Invalid byte sequence: {}", e);
            return None;
            // return none for invalid byte sequence
            //TODO: add checks if the message is not a JSON
        }
    };
    decode_json_message(message)
}

// Parse the json message
fn decode_json_message(message: &str) -> Option<(MexType, String, Vec<u8>)> {
    match serde_json::from_str::<Message>(message) {
        Ok(service_message) => {
            // Extract service name
            let service_name = service_message
                .service
                .split('.')
                .next()
                .unwrap_or("")
                .to_string();
            // Decode base64 payload
            match STANDARD.decode(&service_message.payload) {
                Ok(decoded_payload) => {
                    info!("decoded payload:{:?}", decoded_payload);
                    Some((service_message.direction, service_name, decoded_payload))
                }
                Err(e) => {
                    error!("Invalid Payload: {}", e);
                    None
                }
            }
        }
        Err(e) => {
            error!("Cannot decode JSON message: {:?}", e);
            None
        }
    }
}

// Create JSON message
pub fn create_message(service: &str, direction: MexType, payload: &[u8]) -> Vec<u8> {
    let message = Message {
        direction,
        payload: STANDARD.encode(payload),
        service: service.to_string(),
    };
    match serde_json::to_string(&message) {
        Ok(json) => json.into_bytes(),
        Err(e) => {
            error!("Cannot serialize the message: {}", e);
            Vec::new() // Empty vector in case of error
        }
    }
}
//tcp connection method
//TODO: debug tcp connection
pub async fn send_outcoming_message(stream: &mut TcpStream, service_name: String) {
    info!("Producing outcoming message");
    info!(
        "([{}]->[{:?}]): Receiving outgoing message from: {}",
        service_name,
        stream.peer_addr(),
        service_name
    );

    // Send a response back
    let response_json = json!({ "status": "received" }).to_string();
    if let Err(e) = stream.write_all(response_json.as_bytes()).await {
        error!("Error sending JSON response to {}: {}", service_name, e);
    }
}
//udp connection method
pub async fn send_outcoming_message_udp(
    socket: &UdpSocket,
    service_name: String,
    addr: std::net::SocketAddr,
) -> Vec<u8> {
    info!(
        "([{}]->[{}]):Receiving outgoing message from: {}",
        service_name, addr, service_name
    );

    // Send a response back
    let response_json = json!({ "status": "received" }).to_string();
    if let Err(e) = socket.send_to(&response_json.as_bytes(), addr).await {
        error!(
            "([{}]->[{}]):Error sending JSON response to {}: {}",
            addr, service_name, service_name, e
        );
    }
    response_json.as_bytes().to_vec()
}
//tcp connection method
//TODO: debug this method
pub async fn produce_unknown_message(stream: &mut TcpStream, service_name: String) {
    warn!("Producing message from unknown direction");
    warn!(
        "Receiving message with unknown direction from {}",
        service_name
    );
    warn!("Ignoring the message with unknown direction");

    // Send a response back
    let response_json = json!({ "status": "received" }).to_string();
    if let Err(e) = stream.write_all(response_json.as_bytes()).await {
        error!("Error sending JSON response to {}: {}", service_name, e);
    }
}
//udp connection method
pub async fn produce_unknown_message_udp(
    socket: &UdpSocket,
    service_name: String,
    addr: std::net::SocketAddr,
) -> Vec<u8> {
    warn!(
        "Receiving message with unknown direction from {}",
        service_name
    );
    warn!("Ignoring the message with unknown direction");

    // Send a response back
    let response_json = json!({ "status": "received" }).to_string();
    if let Err(e) = socket.send_to(&response_json.as_bytes(), addr).await {
        error!("Error sending JSON response to {}: {}", service_name, e);
    }
    response_json.as_bytes().to_vec()
}

//tcp connection method
//TODO: debug this method
pub async fn produce_incoming_message(stream: &mut TcpStream, service_name: String) {
    info!("Producing Incoming response message");
    // return a status response
    let response_json = json!({"status":"received"}).to_string();
    info!(
        "Sending TCP response back to {} with content {}",
        service_name, response_json
    );
    let response_message =
        create_message(&service_name, MexType::Outcoming, response_json.as_bytes());

    if let Err(e) = stream.write_all(&response_message).await {
        error!("Error sending {:?} to {}", response_message, service_name);
        error!("Error: {}", e);
    }
}
//tcp connection method
//TODO: debug this method
pub async fn send_success_ack_message(stream: &mut TcpStream) {
    // ACK message
    let ack_message = b"Message Received";
    if let Err(e) = stream.write_all(ack_message).await {
        error!("Error sending TCP acknowledgment: {}", e);
    }
}
//tcp connection method
//TODO: debug this method
pub async fn send_fail_ack_message(stream: &mut TcpStream) {
    // ACK message
    let ack_message = b"Delivery failed";
    if let Err(e) = stream.write_all(ack_message).await {
        error!("Error sending TCP acknowledgment: {}", e);
    }
}
//tcp connnection method
//TODO: debug this method
pub fn ignore_message_with_no_service(direction: MexType, payload: &[u8]) {
    info!(
        "Ignoring message with direction {:?}: {:?}",
        direction, payload
    );
}
