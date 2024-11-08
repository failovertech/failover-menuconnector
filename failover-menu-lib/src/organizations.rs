use crate::client::OpenApiClient;

use serde::{Deserialize, Serialize};
use anyhow::{Result, Context};
use tokio;
use std::env;

use log::{debug, error, info};

#[derive(Debug, Deserialize)]
pub struct Organization {
    #[serde(rename = "responseType")]
    pub response_type: String,
    pub id: String,
    pub name: String,
    #[serde(rename = "country")]
    pub country: Option<String>,
    #[serde(rename = "restaurantAddress")]
    pub restaurant_address: Option<String>,
    #[serde(rename = "useUaeAddressingSystem")]
    pub use_uae_addressing_system: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct OrganizationsResponse {
    pub organizations: Vec<Organization>,
}

pub async fn fetch_organizations(client: &OpenApiClient) -> Result<OrganizationsResponse> {
    client.get("api/1/organizations")
        .await
        .context("Failed to fetch organizations")
}

pub fn print_organization(org: &Organization) {
    println!("  Name: {}", org.name);
    println!("  ID: {}", org.id);
    println!("  Type: {}", org.response_type);
    
    if let Some(country) = &org.country {
        println!("  Country: {}", country);
    }
    
    if let Some(address) = &org.restaurant_address {
        println!("  Address: {}", address);
    }    
}

pub fn print_organizations_response(organizations: Result<OrganizationsResponse>) {
    match organizations {
        Ok(response) => {
            println!("\nSuccessfully retrieved {} organizations:", response.organizations.len());
            
            if response.organizations.is_empty() {
                println!("No organizations found.");
                return;
            }

            for (index, org) in response.organizations.iter().enumerate() {
                println!("\nOrganization {}:", index + 1);    
                print_organization(org);
                println!("\n===================================================");
            }

            return;
        },
        Err(e) => {
            eprintln!("Error fetching organizations:");
            eprintln!("  {:#}", e);
            
            // Print cause chain
            let mut error = e.source();
            while let Some(cause) = error {
                eprintln!("  Caused by: {}", cause);
                error = cause.source();
            }
            
            return;
        }
    }
}

pub fn get_organizations(organizations_response: Result<OrganizationsResponse>) -> Vec<Organization> {
    match organizations_response {
        Ok(response) => {
            return response.organizations
        },
        Err(e) => {
            return vec!();
        }
    }
}

pub fn get_main_organization(organizations_response: &OrganizationsResponse) -> Option<&Organization> {
    organizations_response.organizations.first()
}