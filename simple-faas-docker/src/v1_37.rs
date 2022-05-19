#![allow(non_snake_case)]

use crate::auth::DockerConfig;
use crate::client::Client;
use anyhow::bail;
use log::debug;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Api {
    client: Client,
    docker_config: DockerConfig,
}

impl Api {
    pub fn new(client: Client, docker_config: DockerConfig) -> Self {
        Api {
            client,
            docker_config,
        }
    }

    pub fn images(&self) -> Images {
        Images { api: self.clone() }
    }

    pub fn containers(&self) -> Containers {
        Containers { api: self.clone() }
    }

    pub fn container(&self, id: String) -> Container {
        Container {
            id,
            api: self.clone(),
        }
    }
}

pub struct Images {
    api: Api,
}

#[derive(Debug, PartialEq)]
struct Image {
    domain: String,
    name: String,
    tag: String,
    digest: Option<String>,
}

impl From<Image> for ImageCreateArgs {
    fn from(image: Image) -> Self {
        if image.digest.is_some() {
            unimplemented!("Image digest conversion to image create args is not implemented!");
        }

        Self {
            fromImage: format!("{}/{}", image.domain, image.name),
            tag: image.tag,
        }
    }
}

fn normalize_image_tag(tag: String) -> anyhow::Result<Image> {
    let mut remaining_tag = tag;

    if remaining_tag.is_empty() {
        bail!("Docker image tag cannot be empty");
    }

    let mut image_digest: Option<String> = None;
    if remaining_tag.contains("@") {
        let mut digest_iterator = remaining_tag.rsplit("@");
        image_digest = digest_iterator.next().map(str::to_owned);
        remaining_tag = digest_iterator.collect();
    }

    let image_tag: String;
    if remaining_tag.contains(":") {
        let mut tag_iterator = remaining_tag.rsplit(":");
        image_tag = tag_iterator
            .next()
            .map(str::to_owned)
            .expect("String contains ':' for sure");
        remaining_tag = tag_iterator.collect();
    } else {
        image_tag = "latest".to_string();
    }

    if remaining_tag.matches("/").count() == 0 {
        remaining_tag = format!("library/{}", remaining_tag);
    }

    if remaining_tag.matches("/").count() == 1 {
        remaining_tag = format!("docker.io/{}", remaining_tag);
    }

    if !remaining_tag.starts_with("http://") && !remaining_tag.starts_with("https://") {
        remaining_tag = format!("https://{}", remaining_tag);
    }

    let url = Url::parse(&remaining_tag)?;
    let image_domain = match url.domain() {
        Some(d) => d.to_string(),
        None => "docker.io".to_string(),
    };
    let mut image_name = match url.path().is_empty() {
        true => bail!("Image name cannot be empty"),
        false => url.path().to_string(),
    };

    if image_name.starts_with("/") {
        image_name.remove(0);
    }

    Ok(Image {
        domain: image_domain,
        name: image_name,
        tag: image_tag,
        digest: image_digest,
    })
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct ImageCreateArgs {
    pub fromImage: String,
    pub tag: String,
}

impl Images {
    pub async fn create(&self, body: ImageCreateArgs) -> anyhow::Result<()> {
        let image = normalize_image_tag(body.fromImage.clone())?;
        let auth_entry = self.api.docker_config.auths.get(&image.domain);

        let url = format!(
            "{}/images/create?fromImage={}&tag={}",
            self.api.client.host(),
            body.fromImage,
            body.tag
        );
        let client = reqwest::Client::new();
        let mut request = client.post(url).header("Content-Type", "application/json");

        if let Some(auth) = auth_entry {
            debug!(
                "Providing token auth for domain {} for {}",
                &image.domain, &body.fromImage
            );
            let json = serde_json::to_string(auth)?;
            let base64 = base64::encode(json);

            request = request.header("X-Registry-Auth", base64);
        }

        let response = request.send().await?;
        let status = response.status();
        if status != 200 {
            bail!(
                "Failed to create image: {} ({})",
                response.text().await?,
                status
            );
        }

        Ok(())
    }

    pub async fn pull(&self, tag: String) -> anyhow::Result<()> {
        let image = normalize_image_tag(tag)?;
        self.create(image.into()).await?;

        Ok(())
    }
}

pub struct Containers {
    api: Api,
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
        let url = format!("{}/containers/create", self.api.client.host());
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
            bail!(
                "Failed to create container: {} ({})",
                response.text().await?,
                status
            );
        }

        let response_text = response.text().await?;
        let container_create_response: ContainerCreateResponse =
            serde_json::from_str(&response_text)?;

        Ok(Container {
            id: container_create_response.Id,
            api: self.api.clone(),
        })
    }
}

#[derive(Debug)]
pub struct Container {
    id: String,
    api: Api,
}

impl Container {
    pub async fn start(&self) -> anyhow::Result<()> {
        let url = format!("{}/containers/{}/start", self.api.client.host(), self.id);
        let client = reqwest::Client::new();
        let response = client
            .post(url)
            .header("Content-Type", "application/json")
            .send()
            .await?;

        let status = response.status();
        if status != 204 {
            bail!(
                "Failed to start container: {} ({})",
                response.text().await?,
                status
            );
        }

        // TODO: Check body
        Ok(())
    }

    /// Wait for container to stop
    pub async fn wait(&self) -> anyhow::Result<()> {
        let url = format!("{}/containers/{}/wait", self.api.client.host(), self.id);
        let client = reqwest::Client::new();
        let response = client
            .post(url)
            .header("Content-Type", "application/json")
            .send()
            .await?;

        let status = response.status();
        if status != 200 {
            bail!(
                "Failed to wait for container: {} ({})",
                response.text().await?,
                status
            );
        }

        // TODO: Check body
        Ok(())
    }

    /// Wait for container to stop
    pub async fn delete(&self) -> anyhow::Result<()> {
        let url = format!("{}/containers/{}", self.api.client.host(), self.id);
        let client = reqwest::Client::new();
        let response = client
            .delete(url)
            .header("Content-Type", "application/json")
            .send()
            .await?;

        let status = response.status();
        if status != 204 {
            bail!(
                "Failed to delete {} container: {} ({})",
                self.id,
                response.text().await?,
                status
            );
        }

        // TODO: Check body
        Ok(())
    }

    pub async fn logs(&self) -> anyhow::Result<String> {
        let url = format!(
            "{}/containers/{}/logs?stdout=true",
            self.api.client.host(),
            self.id
        );
        let client = reqwest::Client::new();
        let response = client.get(url).send().await?;

        let status = response.status();
        let body = response.text().await?;
        if status != 200 {
            bail!(
                "Failed to get {} container logs: {} ({})",
                self.id,
                body,
                status
            );
        }

        // TODO: Check body
        Ok(body)
    }
}

#[cfg(test)]
mod tests {
    use super::normalize_image_tag;
    use super::{Image, ImageCreateArgs};

    #[test]
    fn test_normalize_image_tag() {
        assert_eq!(
            normalize_image_tag("ghcr.io/library/hello-world:alpine@1234".into()).unwrap(),
            Image {
                domain: "ghcr.io".into(),
                name: "library/hello-world".into(),
                tag: "alpine".into(),
                digest: Some("1234".into()),
            }
        );

        assert_eq!(
            normalize_image_tag("ghcr.io/library/hello-world:alpine".into()).unwrap(),
            Image {
                domain: "ghcr.io".into(),
                name: "library/hello-world".into(),
                tag: "alpine".into(),
                digest: None,
            }
        );

        assert_eq!(
            normalize_image_tag("ghcr.io/library/hello-world".into()).unwrap(),
            Image {
                domain: "ghcr.io".into(),
                name: "library/hello-world".into(),
                tag: "latest".into(),
                digest: None,
            }
        );

        assert_eq!(
            normalize_image_tag("library/hello-world".into()).unwrap(),
            Image {
                domain: "docker.io".into(),
                name: "library/hello-world".into(),
                tag: "latest".into(),
                digest: None,
            }
        );

        assert_eq!(
            normalize_image_tag("hello-world".into()).unwrap(),
            Image {
                domain: "docker.io".into(),
                name: "library/hello-world".into(),
                tag: "latest".into(),
                digest: None,
            }
        );

        assert!(normalize_image_tag("".into()).is_err());
    }

    #[test]
    fn test_image_create_args_from_image() {
        assert_eq!(
            ImageCreateArgs::from(Image {
                domain: "ghcr.io".into(),
                name: "library/hello-world".into(),
                tag: "alpine".into(),
                digest: None,
            }),
            ImageCreateArgs {
                fromImage: "ghcr.io/library/hello-world".into(),
                tag: "alpine".into(),
            }
        );
    }
}
