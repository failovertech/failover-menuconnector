use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Debug, Serialize, Deserialize)]
struct AuthResponse {
    token: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Organization {
    id: String,
    name: String,
    #[serde(rename = "restaurantAddress")]
    restaurant_address: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct OrganizationsResponse {
    organizations: Vec<Organization>,
}

#[derive(Debug)]
pub struct IikoClient {
    client: Client,
    base_url: String,
    api_login: String,
    access_token: Option<String>,
}

impl IikoClient {
    pub fn new(api_login: String, access_token: String) -> Self {
        Self {
            client: Client::new(),
            base_url: "https://api-ru.iiko.services".to_string(),
            api_login,
            access_token: Some(access_token),
        }
    }

    pub async fn authenticate(&mut self) -> Result<(), Box<dyn Error>> {
        let url = format!("{}/api/1/access_token", self.base_url);
        
        let payload = serde_json::json!({
            "apiLogin": self.api_login
        });

        let response = self.client
            .post(&url)
            .json(&payload)
            .send()
            .await?;

        let auth_response: AuthResponse = response.json().await?;
        self.access_token = Some(auth_response.token);
        
        Ok(())
    }

    pub async fn get_organizations(&self) -> Result<Vec<Organization>, Box<dyn Error>> {
        let token = self.access_token.as_ref()
            .ok_or("Not authenticated. Call authenticate() first")?;
            
        let url = format!("{}/api/1/organizations", self.base_url);

        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .send()
            .await?;

        let organizations_response: OrganizationsResponse = response.json().await?;
        Ok(organizations_response.organizations)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Create Cargo.toml dependencies:
    // [dependencies]
    // reqwest = { version = "0.11", features = ["json"] }
    // serde = { version = "1.0", features = ["derive"] }
    // serde_json = "1.0"
    // tokio = { version = "1.0", features = ["full"] }

    // Replace with your actual API login
    let api_login = "failovermenu1".to_string();
    let access_token = "6c7c5aa5c7f34ce88056603981364f92".to_string();
    
    // Create and authenticate client
    let mut client = IikoClient::new(api_login, access_token);
    client.authenticate().await?;
    
    // Get organizations
    match client.get_organizations().await {
        Ok(organizations) => {
            println!("Organizations:");
            for org in organizations {
                println!("ID: {}", org.id);
                println!("Name: {}", org.name);
                if let Some(address) = org.restaurant_address {
                    println!("Address: {}", address);
                }
                println!("---");
            }
        }
        Err(e) => println!("Error getting organizations: {}", e),
    }

    Ok(())
}