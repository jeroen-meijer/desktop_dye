use anyhow::*;
use constants::*;
use reqwest::{
    header::{HeaderMap, HeaderName},
    Client, Method, Response,
};
use serde_json::Value;
use sprintf::sprintf;

mod constants;

#[derive(Debug)]
pub enum ApiStatus {
    Ok,
    InvalidPassword,
    CannotConnect,
    Unknown,
}

pub type DataMap = std::collections::HashMap<String, Value>;

pub struct HomeAssistantConfig {
    base_url: String,
    token: String,
}

impl HomeAssistantConfig {
    pub fn new(base_url: String, token: String) -> Self {
        Self { base_url, token }
    }
}

pub struct HomeAssistantApi {
    base_url: String,
    headers: HeaderMap,
    client: Client,
}

impl HomeAssistantApi {
    pub fn new(config: &HomeAssistantConfig) -> Self {
        let mut headers = HeaderMap::new();
        headers.insert(
            "content-type".parse::<HeaderName>().unwrap(),
            "application/json".parse().unwrap(),
        );
        headers.insert(
            "Authorization".parse::<HeaderName>().unwrap(),
            format!("Bearer {}", config.token).parse().unwrap(),
        );

        Self {
            base_url: config.base_url.clone(),
            headers,
            client: Client::new(),
        }
    }

    async fn request(
        &self,
        method: Method,
        path: String,
        data: Option<DataMap>,
    ) -> Result<Response> {
        let uri = format!("{}{}", self.base_url, path);

        let mut request_builder = self.client.request(method, &uri);

        if let Some(data) = data {
            request_builder = request_builder.json(&data);
        }
        let request = request_builder
            .headers(self.headers.clone())
            .build()
            .context(format!("Error building request for {}", uri))?;

        let response = self
            .client
            .execute(request)
            .await
            .context(format!("Error executing request for {}", uri))?;
        Ok(response)
    }

    pub async fn get_status(&self) -> Result<ApiStatus> {
        let response = self
            .request(Method::GET, BASE_URL.to_string(), None)
            .await?;
        match response.status().as_u16() {
            200 => Ok(ApiStatus::Ok),
            401 => Ok(ApiStatus::InvalidPassword),
            _ => Ok(ApiStatus::Unknown),
        }
    }

    pub async fn set_state(
        &self,
        entity_id: String,
        new_state: String,
        attributes: Option<DataMap>,
        force_update: bool,
    ) -> Result<()> {
        let mut data = DataMap::new();
        data.insert("state".to_string(), Value::String(new_state));

        if let Some(attributes) = attributes {
            data.insert(
                "attributes".to_string(),
                serde_json::Map::from_iter(attributes.into_iter()).into(),
            );
        }

        data.insert("force_update".to_string(), Value::Bool(force_update));

        let response = self
            .request(
                Method::POST,
                sprintf!(URL_STATES_ENTITY, entity_id).unwrap(),
                Some(data),
            )
            .await?;
        match response.status().as_u16() {
            200 => Ok(()),
            code => Err(anyhow!("Error setting state: {}", code)),
        }
    }

    pub async fn get_state(&self, entity_id: String) -> Result<Value> {
        Ok(self
            .request(
                Method::GET,
                sprintf!(URL_STATES_ENTITY, entity_id).unwrap(),
                None,
            )
            .await?
            .json::<Value>()
            .await?)
    }

    pub async fn get_states(&self) -> Result<Value> {
        Ok(self
            .request(Method::GET, URL_STATES.to_string(), None)
            .await?
            .json::<Value>()
            .await?)
    }

    pub async fn get_services(&self) -> Result<Value> {
        Ok(self
            .request(Method::GET, URL_SERVICES.to_string(), None)
            .await?
            .json::<Value>()
            .await?)
    }

    pub async fn is_state(&self, entity_id: String, state: String) -> Result<bool> {
        let response = self.get_state(entity_id).await?;
        Ok(response["state"].as_str().unwrap() == state.as_str())
    }

    pub async fn call_services(
        &self,
        domain: String,
        service: String,
        data: Option<DataMap>,
    ) -> Result<()> {
        let response = self
            .request(
                Method::POST,
                sprintf!(URL_SERVICES_SERVICE, domain, service.clone()).unwrap(),
                data,
            )
            .await?;

        match response.status().as_u16() {
            200 => Ok(()),
            code => Err(anyhow!(format!(
                "Error calling service {}: {} - {}",
                service,
                code,
                response.text().await?
            ))),
        }
    }
}
