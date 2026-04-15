use crate::{ApiConfig, ApiResponse, Breeder, BreederCreateRequest, BreederSummary, BreederUpdateRequest, Credential, Target};
use anyhow::{Context, Result};
use reqwest::Client;
use std::time::Duration;

pub struct GodonClient {
    config: ApiConfig,
    client: Client,
    debug: bool,
}

impl GodonClient {
    pub fn new(hostname: String, port: u16, api_version: String, insecure: bool, debug: bool) -> Result<Self> {
        let mut builder = Client::builder()
            .timeout(Duration::from_secs(30));

        if insecure {
            builder = builder.danger_accept_invalid_certs(true);
        }

        let client = builder.build()
            .context("Failed to create HTTP client")?;

        Ok(Self {
            config: ApiConfig {
                hostname,
                port,
                api_version,
            },
            client,
            debug,
        })
    }

    fn base_url(&self) -> String {
        let scheme = if self.config.hostname.starts_with("https://") {
            "https"
        } else if self.config.hostname.starts_with("http://") {
            "http"
        } else {
            "http"
        };

        let clean_host = self.config.hostname
            .trim_start_matches("https://")
            .trim_start_matches("http://");

        format!("{}://{}:{}", scheme, clean_host, self.config.port)
    }

    async fn handle_response<T: serde::de::DeserializeOwned>(&self, response: reqwest::Response) -> ApiResponse<T> {
        let status = response.status();
        
        if status.is_success() {
            match response.text().await {
                Ok(body) => {
                    if self.debug {
                        eprintln!("Raw response body: {}", body);
                    }
                    
                    if body.is_empty() {
                        return ApiResponse::error("Empty response body");
                    }

                    match serde_json::from_str::<T>(&body) {
                        Ok(data) => ApiResponse::success(data),
                        Err(e) => {
                            if self.debug {
                                eprintln!("JSON parse error: {}", e);
                            }
                            ApiResponse::error(format!("JSON parse error: {}", e))
                        }
                    }
                }
                Err(e) => ApiResponse::error(format!("Failed to read response: {}", e)),
            }
        } else {
            match response.text().await {
                Ok(body) => {
                    if self.debug {
                        eprintln!("HTTP Error Response Body: {}", body);
                    }
                    
                    let error_msg = serde_json::from_str::<serde_json::Value>(&body)
                        .ok()
                        .and_then(|v| v.get("message").and_then(|m| m.as_str()).map(String::from))
                        .unwrap_or_else(|| format!("HTTP Error: {}", status));

                    ApiResponse::error(error_msg)
                }
                Err(_) => ApiResponse::error(format!("HTTP Error: {}", status)),
            }
        }
    }

    pub async fn list_breeders(&self) -> ApiResponse<Vec<BreederSummary>> {
        let url = format!("{}/breeders", self.base_url());
        
        match self.client.get(&url).send().await {
            Ok(response) => self.handle_response(response).await,
            Err(e) => ApiResponse::error(e.to_string()),
        }
    }

    pub async fn create_breeder(&self, request: BreederCreateRequest) -> ApiResponse<BreederSummary> {
        let url = format!("{}/breeders", self.base_url());
        
        if self.debug {
            eprintln!("Sending JSON: {}", serde_json::to_string_pretty(&request).unwrap_or_default());
        }

        match self.client
            .post(&url)
            .json(&request)
            .send()
            .await
        {
            Ok(response) => self.handle_response(response).await,
            Err(e) => ApiResponse::error(e.to_string()),
        }
    }

    pub async fn create_breeder_from_yaml(&self, yaml_content: &str, name: &str) -> ApiResponse<BreederSummary> {
        let config: serde_json::Value = match serde_yaml::from_str(yaml_content) {
            Ok(c) => c,
            Err(e) => return ApiResponse::error(format!("YAML parse error: {}", e)),
        };

        let request = BreederCreateRequest {
            name: name.to_string(),
            config,
        };

        self.create_breeder(request).await
    }

    pub async fn get_breeder(&self, uuid: &str) -> ApiResponse<Breeder> {
        let url = format!("{}/breeders/{}", self.base_url(), urlencoding::encode(uuid));
        
        match self.client.get(&url).send().await {
            Ok(response) => self.handle_response(response).await,
            Err(e) => ApiResponse::error(e.to_string()),
        }
    }

    pub async fn update_breeder(&self, request: BreederUpdateRequest) -> ApiResponse<serde_json::Value> {
        let url = format!("{}/breeders/{}", self.base_url(), urlencoding::encode(&request.uuid));
        
        let config: serde_json::Value = match serde_json::from_str(&request.config) {
            Ok(c) => c,
            Err(e) => return ApiResponse::error(format!("Config JSON parse error: {}", e)),
        };

        let body = serde_json::json!({
            "name": request.name,
            "description": request.description,
            "config": config
        });

        match self.client
            .put(&url)
            .json(&body)
            .send()
            .await
        {
            Ok(response) => self.handle_response(response).await,
            Err(e) => ApiResponse::error(e.to_string()),
        }
    }

    pub async fn update_breeder_from_yaml(&self, yaml_content: &str) -> ApiResponse<serde_json::Value> {
        let yaml_data: serde_yaml::Value = match serde_yaml::from_str(yaml_content) {
            Ok(d) => d,
            Err(e) => return ApiResponse::error(format!("YAML parse error: {}", e)),
        };

        let uuid = yaml_data.get("uuid")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let name = yaml_data.get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let description = yaml_data.get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let config = yaml_data.get("config")
            .map(|c| serde_json::to_string(c).unwrap_or_else(|_| "{}".to_string()))
            .unwrap_or_else(|| "{}".to_string());

        let request = BreederUpdateRequest {
            uuid,
            name,
            description,
            config,
        };

        self.update_breeder(request).await
    }

    pub async fn delete_breeder(&self, uuid: &str, force: bool) -> ApiResponse<serde_json::Value> {
        let mut url = format!("{}/breeders/{}", self.base_url(), urlencoding::encode(uuid));
        
        if force {
            url.push_str("?force=true");
        }

        match self.client.delete(&url).send().await {
            Ok(response) => self.handle_response(response).await,
            Err(e) => ApiResponse::error(e.to_string()),
        }
    }

    pub async fn stop_breeder(&self, uuid: &str) -> ApiResponse<serde_json::Value> {
        let url = format!("{}/breeders/{}/stop", self.base_url(), urlencoding::encode(uuid));
        
        match self.client.post(&url).send().await {
            Ok(response) => self.handle_response(response).await,
            Err(e) => ApiResponse::error(e.to_string()),
        }
    }

    pub async fn start_breeder(&self, uuid: &str) -> ApiResponse<serde_json::Value> {
        let url = format!("{}/breeders/{}/start", self.base_url(), urlencoding::encode(uuid));
        
        match self.client.post(&url).send().await {
            Ok(response) => self.handle_response(response).await,
            Err(e) => ApiResponse::error(e.to_string()),
        }
    }

    pub async fn list_credentials(&self) -> ApiResponse<Vec<Credential>> {
        let url = format!("{}/credentials", self.base_url());
        
        match self.client.get(&url).send().await {
            Ok(response) => {
                let status = response.status();
                
                if status.is_success() {
                    match response.text().await {
                        Ok(body) => {
                            let json: serde_json::Value = match serde_json::from_str(&body) {
                                Ok(j) => j,
                                Err(e) => return ApiResponse::error(format!("JSON parse error: {}", e)),
                            };

                            let credentials: Vec<Credential> = if json.is_array() {
                                match serde_json::from_value(json) {
                                    Ok(c) => c,
                                    Err(e) => return ApiResponse::error(format!("Parse error: {}", e)),
                                }
                            } else if let Some(arr) = json.get("credentials") {
                                match serde_json::from_value(arr.clone()) {
                                    Ok(c) => c,
                                    Err(e) => return ApiResponse::error(format!("Parse error: {}", e)),
                                }
                            } else {
                                return ApiResponse::error("Unexpected response format");
                            };

                            ApiResponse::success(credentials)
                        }
                        Err(e) => ApiResponse::error(e.to_string()),
                    }
                } else {
                    ApiResponse::error(format!("HTTP Error: {}", status))
                }
            }
            Err(e) => ApiResponse::error(e.to_string()),
        }
    }

    pub async fn create_credential(&self, credential_data: serde_json::Value) -> ApiResponse<Credential> {
        let url = format!("{}/credentials", self.base_url());
        
        match self.client
            .post(&url)
            .json(&credential_data)
            .send()
            .await
        {
            Ok(response) => {
                let status = response.status();
                
                if status.is_success() {
                    match response.text().await {
                        Ok(body) => {
                            match serde_json::from_str::<Credential>(&body) {
                                Ok(cred) => ApiResponse::success(cred),
                                Err(e) => ApiResponse::error(format!("Parse error: {}", e)),
                            }
                        }
                        Err(e) => ApiResponse::error(e.to_string()),
                    }
                } else {
                    ApiResponse::error(format!("HTTP Error: {}", status))
                }
            }
            Err(e) => ApiResponse::error(e.to_string()),
        }
    }

    pub async fn get_credential(&self, credential_id: &str) -> ApiResponse<Credential> {
        let url = format!("{}/credentials/{}", self.base_url(), urlencoding::encode(credential_id));
        
        match self.client.get(&url).send().await {
            Ok(response) => {
                let status = response.status();
                
                if status.is_success() {
                    match response.text().await {
                        Ok(body) => {
                            match serde_json::from_str::<Credential>(&body) {
                                Ok(cred) => ApiResponse::success(cred),
                                Err(e) => ApiResponse::error(format!("Parse error: {}", e)),
                            }
                        }
                        Err(e) => ApiResponse::error(e.to_string()),
                    }
                } else {
                    ApiResponse::error(format!("HTTP Error: {}", status))
                }
            }
            Err(e) => ApiResponse::error(e.to_string()),
        }
    }

    pub async fn delete_credential(&self, credential_id: &str) -> ApiResponse<serde_json::Value> {
        let url = format!("{}/credentials/{}", self.base_url(), urlencoding::encode(credential_id));
        
        match self.client.delete(&url).send().await {
            Ok(response) => {
                let status = response.status();
                
                if status.is_success() {
                    match response.text().await {
                        Ok(body) => {
                            match serde_json::from_str(&body) {
                                Ok(v) => ApiResponse::success(v),
                                Err(e) => ApiResponse::error(format!("Parse error: {}", e)),
                            }
                        }
                        Err(e) => ApiResponse::error(e.to_string()),
                    }
                } else {
                    ApiResponse::error(format!("HTTP Error: {}", status))
                }
            }
            Err(e) => ApiResponse::error(e.to_string()),
        }
    }

    pub async fn create_credential_from_yaml(&self, yaml_content: &str) -> ApiResponse<Credential> {
        let yaml_data: std::collections::HashMap<String, String> = match serde_yaml::from_str(yaml_content) {
            Ok(d) => d,
            Err(e) => return ApiResponse::error(format!("YAML parse error: {}", e)),
        };

        let name = match yaml_data.get("name") {
            Some(n) => n.clone(),
            None => return ApiResponse::error("Missing required field: name"),
        };

        let credential_type = match yaml_data.get("credentialType") {
            Some(t) => t.clone(),
            None => return ApiResponse::error("Missing required field: credentialType"),
        };

        let content = match yaml_data.get("content") {
            Some(c) => c.clone(),
            None => return ApiResponse::error("Missing required field: content"),
        };

        let description = yaml_data.get("description").cloned().unwrap_or_default();

        let credential_data = serde_json::json!({
            "name": name,
            "credentialType": credential_type,
            "description": description,
            "content": content
        });

        self.create_credential(credential_data).await
    }

    pub async fn list_targets(&self) -> ApiResponse<Vec<Target>> {
        let url = format!("{}/targets", self.base_url());

        match self.client.get(&url).send().await {
            Ok(response) => {
                let status = response.status();

                if status.is_success() {
                    match response.text().await {
                        Ok(body) => {
                            let json: serde_json::Value = match serde_json::from_str(&body) {
                                Ok(j) => j,
                                Err(e) => return ApiResponse::error(format!("JSON parse error: {}", e)),
                            };

                            let targets: Vec<Target> = if json.is_array() {
                                match serde_json::from_value(json) {
                                    Ok(t) => t,
                                    Err(e) => return ApiResponse::error(format!("Parse error: {}", e)),
                                }
                            } else if let Some(arr) = json.get("targets") {
                                match serde_json::from_value(arr.clone()) {
                                    Ok(t) => t,
                                    Err(e) => return ApiResponse::error(format!("Parse error: {}", e)),
                                }
                            } else {
                                return ApiResponse::error("Unexpected response format");
                            };

                            ApiResponse::success(targets)
                        }
                        Err(e) => ApiResponse::error(e.to_string()),
                    }
                } else {
                    ApiResponse::error(format!("HTTP Error: {}", status))
                }
            }
            Err(e) => ApiResponse::error(e.to_string()),
        }
    }

    pub async fn create_target(&self, target_data: serde_json::Value) -> ApiResponse<Target> {
        let url = format!("{}/targets", self.base_url());

        match self.client
            .post(&url)
            .json(&target_data)
            .send()
            .await
        {
            Ok(response) => {
                let status = response.status();

                if status.is_success() {
                    match response.text().await {
                        Ok(body) => {
                            match serde_json::from_str::<Target>(&body) {
                                Ok(target) => ApiResponse::success(target),
                                Err(e) => ApiResponse::error(format!("Parse error: {}", e)),
                            }
                        }
                        Err(e) => ApiResponse::error(e.to_string()),
                    }
                } else {
                    ApiResponse::error(format!("HTTP Error: {}", status))
                }
            }
            Err(e) => ApiResponse::error(e.to_string()),
        }
    }

    pub async fn get_target(&self, target_id: &str) -> ApiResponse<Target> {
        let url = format!("{}/targets/{}", self.base_url(), urlencoding::encode(target_id));

        match self.client.get(&url).send().await {
            Ok(response) => {
                let status = response.status();

                if status.is_success() {
                    match response.text().await {
                        Ok(body) => {
                            match serde_json::from_str::<Target>(&body) {
                                Ok(target) => ApiResponse::success(target),
                                Err(e) => ApiResponse::error(format!("Parse error: {}", e)),
                            }
                        }
                        Err(e) => ApiResponse::error(e.to_string()),
                    }
                } else {
                    ApiResponse::error(format!("HTTP Error: {}", status))
                }
            }
            Err(e) => ApiResponse::error(e.to_string()),
        }
    }

    pub async fn delete_target(&self, target_id: &str) -> ApiResponse<serde_json::Value> {
        let url = format!("{}/targets/{}", self.base_url(), urlencoding::encode(target_id));

        match self.client.delete(&url).send().await {
            Ok(response) => {
                let status = response.status();

                if status.is_success() {
                    match response.text().await {
                        Ok(body) => {
                            match serde_json::from_str(&body) {
                                Ok(v) => ApiResponse::success(v),
                                Err(e) => ApiResponse::error(format!("Parse error: {}", e)),
                            }
                        }
                        Err(e) => ApiResponse::error(e.to_string()),
                    }
                } else {
                    ApiResponse::error(format!("HTTP Error: {}", status))
                }
            }
            Err(e) => ApiResponse::error(e.to_string()),
        }
    }

    pub async fn create_target_from_yaml(&self, yaml_content: &str) -> ApiResponse<Target> {
        let yaml_data: std::collections::HashMap<String, serde_yaml::Value> = match serde_yaml::from_str(yaml_content) {
            Ok(d) => d,
            Err(e) => return ApiResponse::error(format!("YAML parse error: {}", e)),
        };

        let name = match yaml_data.get("name").and_then(|v| v.as_str()) {
            Some(n) => n.to_string(),
            None => return ApiResponse::error("Missing required field: name"),
        };

        let target_type = match yaml_data.get("targetType").and_then(|v| v.as_str()) {
            Some(t) => t.to_string(),
            None => return ApiResponse::error("Missing required field: targetType"),
        };

        let spec = match yaml_data.get("spec") {
            Some(s) => serde_json::to_value(s).unwrap_or_else(|_| serde_json::json!({})),
            None => return ApiResponse::error("Missing required field: spec"),
        };

        let mut target_data = serde_json::json!({
            "name": name,
            "targetType": target_type,
            "spec": spec
        });

        if let Some(m) = yaml_data.get("metadata") {
            target_data["metadata"] = serde_json::to_value(m).unwrap_or_else(|_| serde_json::json!({}));
        }

        self.create_target(target_data).await
    }
}
