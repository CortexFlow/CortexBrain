// src/loadbalancer/loadbalancer.rs

use reqwest::Client; // Assicurati di avere reqwest importato
use std::error::Error;

pub struct LoadBalancer {
    servers: Vec<String>, // Elenco dei server
    client: Client, // Client HTTP
}

impl LoadBalancer {
    pub fn new(servers: Vec<String>) -> Self {
        LoadBalancer {
            servers,
            client: Client::new(), // Inizializza il client
        }
    }

    pub async fn handle_request(&self) -> Result<String, Box<dyn Error>> {
        // In questo esempio, inviamo una richiesta al primo server
        let response = self.client.get(&self.servers[0]).send().await?;

        // Controlla se la risposta è valida
        let response_body = response.text().await?;
        Ok(response_body) // Restituisce il corpo della risposta
    }
}
