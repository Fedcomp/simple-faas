#![allow(non_snake_case)]

use serde::{Deserialize, Serialize};
use anyhow::bail;
use crate::client::Client;

pub struct Api {
    client: Client
}

impl Api {
    pub fn new(client: Client) -> Self {
        Api { client }
    }

    pub fn containers(&self) -> Containers {
        Containers { client: self.client.clone() }
    }

    pub fn container(&self, id: String) -> Container {
        Container {
            id,
            client: self.client.clone()
        }
    }
}

pub struct Containers {
    client: Client
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct ContainerCreateArgs {
    pub Image: String,
    pub Cmd: Option<String>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct ContainerCreateResponse {
    pub Id: String,
}

impl Containers {
    pub async fn create(&self, body: ContainerCreateArgs) -> anyhow::Result<Container> {
        let url = format!("{}/containers/create", self.client.host());
        let raw_body = serde_json::to_string(&body)?;
        let client = reqwest::Client::new();
        let response = client
            .post(url)
            .body(raw_body)
            .header("Content-Type", "application/json")
            .send()
            .await?;

        let status = response.status();
        if status != 201 {
            bail!("Failed to create container: {} ({})", response.text().await?, status);
        }

        let response_text = response.text().await?;
        let container_create_response: ContainerCreateResponse = serde_json::from_str(&response_text)?;

        Ok(Container {
            id: container_create_response.Id,
            client: self.client.clone()
        })
    }
}

#[derive(Debug)]
pub struct Container {
    id: String,
    client: Client
}

impl Container {
    pub async fn start(&self) -> anyhow::Result<()> {
        let url = format!("{}/containers/{}/start", self.client.host(), self.id);
        let client = reqwest::Client::new();
        let response = client
            .post(url)
            .header("Content-Type", "application/json")
            .send()
            .await?;

        let status = response.status();
        if status != 204 {
            bail!("Failed to start container: {} ({})", response.text().await?, status);
        }

        // TODO: Check body
        Ok(())
    }

    /// Wait for container to stop
    pub async fn wait(&self) -> anyhow::Result<()> {
        let url = format!("{}/containers/{}/wait", self.client.host(), self.id);
        let client = reqwest::Client::new();
        let response = client
            .post(url)
            .header("Content-Type", "application/json")
            .send()
            .await?;

        let status = response.status();
        if status != 200 {
            bail!("Failed to wait for container: {} ({})", response.text().await?, status);
        }

        // TODO: Check body
        Ok(())
    }

    /// Wait for container to stop
    pub async fn delete(&self) -> anyhow::Result<()> {
        let url = format!("{}/containers/{}", self.client.host(), self.id);
        let client = reqwest::Client::new();
        let response = client
            .delete(url)
            .header("Content-Type", "application/json")
            .send()
            .await?;

        let status = response.status();
        if status != 204 {
            bail!("Failed to delete {} container: {} ({})", self.id, response.text().await?, status);
        }

        // TODO: Check body
        Ok(())
    }

    pub async fn logs(&self) -> anyhow::Result<String> {
        let url = format!("{}/containers/{}/logs?stdout=true", self.client.host(), self.id);
        let client = reqwest::Client::new();
        let response = client
            .get(url)
            .send()
            .await?;

        let status = response.status();
        let body = response.text().await?;
        if status != 200 {
            bail!("Failed to get {} container logs: {} ({})", self.id, body, status);
        }
    
        // TODO: Check body
        Ok(body)
    }
}
