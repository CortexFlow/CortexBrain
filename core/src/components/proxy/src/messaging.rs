/* Contains all the functions used to communicate between services */
use base64::{Engine as _, engine::general_purpose::STANDARD};
use serde::{Deserialize, Serialize};
use tracing::{error, info};

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
    payload: String,
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
