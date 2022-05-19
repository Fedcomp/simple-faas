use anyhow::{anyhow, Context};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::TryFrom;
use std::env::var;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

pub fn read_default() -> anyhow::Result<DockerConfig> {
    let home_dir = var("HOME").with_context(|| "HOME env is not set")?;
    let path = PathBuf::new().join(home_dir).join(".docker/config.json");
    let docker_config = File::open(path).with_context(|| "Failed to open docker config")?;
    read_auth(docker_config)
}

pub fn read_auth<R: Read>(reader: R) -> anyhow::Result<DockerConfig> {
    let raw_config: RawDockerConfig = serde_json::from_reader(reader)?;
    let config: DockerConfig = raw_config.try_into()?;
    Ok(config)
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DockerConfig {
    pub auths: HashMap<String, DockerAuth>,
}

impl TryFrom<RawDockerConfig> for DockerConfig {
    type Error = anyhow::Error;

    fn try_from(raw_docker_config: RawDockerConfig) -> Result<Self, Self::Error> {
        let mut auths = HashMap::new();

        for (domain, raw_auth) in raw_docker_config.auths.into_iter() {
            let auth: DockerAuth = raw_auth.try_into()?;
            auths.insert(domain, auth);
        }

        Ok(DockerConfig { auths })
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DockerAuth {
    pub username: String,
    pub password: String,
}

impl TryFrom<RawDockerAuth> for DockerAuth {
    type Error = anyhow::Error;

    fn try_from(raw_docker_auth: RawDockerAuth) -> Result<Self, Self::Error> {
        let decoded = base64::decode(raw_docker_auth.auth)?;
        let stringified = String::from_utf8_lossy(&decoded);
        let (username, password) = stringified
            .split_once(":")
            .ok_or_else(|| anyhow!("No : delimiter in username/password registry auth token"))?;

        Ok(DockerAuth {
            username: username.into(),
            password: password.into(),
        })
    }
}

#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
struct RawDockerConfig {
    pub auths: HashMap<String, RawDockerAuth>,
}

#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
struct RawDockerAuth {
    pub auth: String,
}

#[cfg(test)]
mod tests {
    use super::{read_auth, DockerAuth, DockerConfig};
    use std::collections::HashMap;
    use std::io::Cursor;

    #[test]
    fn test_read_auth() {
        let source = r#"{
            "auths": {
                "ghcr.io": {
                        "auth": "MTIzOjEyMw=="
                },
                "registry.gitlab.com": {
                        "auth": "MzIxOjMyMQ=="
                }
            }
        }"#;

        let buf = Cursor::new(source);
        let result = read_auth(buf).unwrap();
        let mut expected_auths = HashMap::new();
        expected_auths.insert(
            "ghcr.io".into(),
            DockerAuth {
                username: "123".into(),
                password: "123".into(),
            },
        );
        expected_auths.insert(
            "registry.gitlab.com".into(),
            DockerAuth {
                username: "321".into(),
                password: "321".into(),
            },
        );
        let expected = DockerConfig {
            auths: expected_auths,
        };

        assert_eq!(result, expected);
    }
}
