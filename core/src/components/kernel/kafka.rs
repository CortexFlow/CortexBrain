/* This is where all the kafka functions are stored 

kafka manages the communication between microservices in the service mesh
*/

use prost::Message;
use rdkafka::consumer::{BaseConsumer, Consumer};
use rdkafka::message::BorrowedMessage;
use rdkafka::producer::{BaseProducer, BaseRecord};
use rdkafka::{ClientConfig, Message as _};
//use rdkafka::error::KafkaError;
use std::time::Duration;

mod proxy_message {
    include!("../kernel/proxy_msg.rs");
}

use proxy_message::ProxyMessage;

pub fn send_message(producer: &BaseProducer, topic: &str, message: ProxyMessage) {
    // Serializza il messaggio
    let mut buf = Vec::with_capacity(message.encoded_len());
    message.encode(&mut buf).expect("Failed to encode message");

    producer.send(
        BaseRecord::to(topic)
            .key("proxy_key")
            .payload(&buf), // Conversione implicita a &[u8]
    ).unwrap_or_else(|e| println!("Error sending message: {:?}", e));
}

pub fn consume_and_forward(consumer: &BaseConsumer, producer: &BaseProducer, output_topic: &str) {
    loop {
        match consumer.poll(Duration::from_secs(1)) {
            Some(Ok(borrowed_message)) => handle_message(borrowed_message, producer, output_topic),
            Some(Err(e)) => eprintln!("Kafka error: {:?}", e),
            None => continue,
        }
    }
}

fn handle_message(message: BorrowedMessage, producer: &BaseProducer, output_topic: &str) {
    // Verifica se il payload è presente
    if let Some(payload) = message.payload() {
        // Tentativo di decodifica del messaggio con prost
        match ProxyMessage::decode(payload) {
            Ok(proxy_message) => {
                println!(
                    "Message received from proxy ({} bytes): {:?}",
                    payload.len(),
                    proxy_message
                );

                // Invio del messaggio al producer Kafka
                send_message(producer, output_topic, proxy_message);
            }
            Err(e) => {
                println!("Failed to decode the message: {:?}", e);
            }
        }
    } else {
        println!("Message payload is empty.");
    }
}


pub fn test_kafka() {
    println!("Testing kafka connection");
    // Configura producer
    let producer: BaseProducer = ClientConfig::new()
        .set("bootstrap.servers", "localhost:9092")
        .create()
        .expect("Producer creation error");

    // Configura consumer
    let consumer: BaseConsumer = ClientConfig::new()
        .set("group.id", "proxy_group")
        .set("bootstrap.servers", "localhost:9092")
        .create()
        .expect("Consumer creation error");

    consumer.subscribe(&["input-topic"]).expect("Subscription failed");

    // Esegui il consumo e inoltro dei messaggi
    consume_and_forward(&consumer, &producer, "output-topic");
}
