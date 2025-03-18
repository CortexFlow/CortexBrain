/* Contains all the functions used to communicate between services */
use tracing::{error, info};

/*
Extract the service name and the payload from this format:

               <service_name>.<namespace>:<message>

*/

pub fn extract_service_name_and_payload(msg_encrypted: &[u8]) -> Option<(&str, &[u8])> {
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
        }
    };

    // Seach for ':' delimiter
    if let Some(col_index) = message.find(':') {
        let service_name_with_namespace = &message[..col_index];
        let payload = &msg_encrypted[col_index + 1..];

        // Extract only the name of the service (before the first ‘.’)
        let service_name = service_name_with_namespace.split('.').next().unwrap_or("");
        Some((service_name, payload))
    } else {
        error!("Delimiter ':' not found");
        Some(("", msg_encrypted)) //if the delimiter is not found, consider the msg_encrypted as a full message 
        //TODO: rework the logic to properly filter incoming messages and outcoming messages
    }
}
