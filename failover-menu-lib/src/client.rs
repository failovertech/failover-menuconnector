use reqwest::{Client, StatusCode, header::{HeaderMap, HeaderValue, CONTENT_TYPE}};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use crate::credentials::Credentials;
use anyhow::{Result, Context, anyhow};
use std::time::Duration;
use std::sync::RwLock;

#[derive(Debug, Serialize)]
pub struct GetAccessTokenRequest {
    #[serde(rename = "apiLogin")]
    api_login: String,
}

#[derive(Debug, Deserialize)]
pub struct GetAccessTokenResponse {
    #[serde(rename = "token")]
    token: String,
}

#[derive(Debug, Deserialize)]
pub struct ErrorResponse {
    #[serde(rename = "errorDescription")]
    error_description: String,
    #[serde(rename = "errorCode")]
    error_code: Option<String>,
}

#[derive(Debug)]
pub struct OpenApiClient {
    client: Client,
    credentials: Credentials,
    session_token: RwLock<Option<String>>,
}

impl OpenApiClient {
    pub fn new(credentials: Credentials) -> Result<Self> {
        log::debug!("CREATING CLIENT");

        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self { 
            client, 
            credentials,
            session_token: RwLock::new(None),
        })
    }

    pub async fn authenticate(&self, timeout_seconds: Option<i32>) -> Result<()> {
        log::debug!("RUNNING FULL AUTH");

        let request = GetAccessTokenRequest {
            api_login: self.credentials.key.clone(),
        };

        let url = self.build_url("api/1/access_token");
        
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        
        if let Some(timeout) = timeout_seconds {
            headers.insert(
                "Timeout",
                HeaderValue::from_str(&timeout.to_string())
                    .context("Failed to create Timeout header")?
            );
        }

        let response = self.client
            .post(&url)
            .headers(headers)
            .json(&request)
            .send()
            .await
            .context("Failed to send authentication request")?;

        match response.status() {
            StatusCode::OK => {
                let token_response: GetAccessTokenResponse = response
                    .json()
                    .await
                    .context("Failed to deserialize access token response")?;
                //log::debug!("{:?}",token_response);                
                
                let mut token = self.session_token.write()
                    .map_err(|_| anyhow!("Failed to acquire write lock for session token"))?;
                *token = Some(token_response.token);                
                Ok(())
            },
            status => {
                let error: ErrorResponse = response
                    .json()
                    .await
                    .context("Failed to deserialize error response")?;
                
                Err(anyhow!(
                    "Authentication failed with status {}: {}",
                    status,
                    error.error_description
                ))
            }
        }
    }

    fn build_url(&self, endpoint: &str) -> String {
        //log::debug!("BUILDING URL");
        format!(
            "{}/{}",
            self.credentials.endpoint.trim_end_matches('/'),
            endpoint.trim_start_matches('/')
        )
    }

    fn get_headers(&self) -> Result<HeaderMap> {
        log::debug!("GETTING HEADERS");

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        // Add session token if available
        if let Some(token) = self.session_token.read()
            .map_err(|_| anyhow!("Failed to acquire read lock for session token"))?
            .as_ref() 
        {
            let token_value = format!("Bearer {token}");
            //log::debug!("Auth token: {}", token_value);
            headers.insert(
                "Authorization",
                HeaderValue::from_str(&token_value)
                    .context("Failed to create Authorization header")?,
            );
        }

        Ok(headers)
    }

    pub async fn ensure_authenticated(&self) -> Result<()> {   
        log::debug!("ENSURING AUTH");
     
        if self.session_token.read()
            .map_err(|_| anyhow!("Failed to acquire read lock for session token"))?
            .is_none() 
        {
            self.authenticate(None).await?;
        }
        Ok(())
    }

    pub async fn get<T>(&self, endpoint: &str) -> Result<T>
    where
        T: DeserializeOwned,
    {
        log::debug!("RUNNING GET: {}", endpoint);

        self.ensure_authenticated().await?;

        let url = self.build_url(endpoint);
        let headers = self.get_headers()?;

        let response = self.client
            .get(&url)
            .headers(headers)
            .send()
            .await
            .context("Failed to send GET request")?;

        self.handle_response(response).await
    }

    pub async fn post<T, B>(&self, endpoint: &str, body: &B) -> Result<T>
    where
        T: DeserializeOwned,
        B: Serialize + ?Sized,
    {
        log::debug!("RUNNING POST: {}", endpoint);

        self.ensure_authenticated().await?;

        let url = self.build_url(endpoint);
        let headers = self.get_headers()?;

        let response = self.client
            .post(&url)
            .headers(headers)
            .json(body)
            .send()
            .await
            .context("Failed to send POST request")?;

        self.handle_response(response).await
    }

    async fn handle_response<T>(&self, response: reqwest::Response) -> Result<T>
    where
        T: DeserializeOwned,
    {
        log::debug!("HANDLING RESPONSE");

        match response.status() {
            StatusCode::OK => {
                response.json().await.context("Failed to deserialize response body")
            },
            StatusCode::UNAUTHORIZED => {
                // Clear the session token and retry once
                {
                    let mut token = self.session_token.write()
                        .map_err(|_| anyhow!("Failed to acquire write lock for session token"))?;
                    *token = None;
                }
                Err(anyhow!("Authentication failed, please retry the request"))
            },
            status => {
                let error: ErrorResponse = response
                    .json()
                    .await
                    .context("Failed to deserialize error response")?;
                
                Err(anyhow!(
                    "Request failed with status {}: {}",
                    status,
                    error.error_description
                ))
            }
        }
    }
}

