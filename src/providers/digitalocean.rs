use log::{debug, info};
use serde::Deserialize;
use std::convert::TryFrom;
use std::env;

use crate::{args::ParsedArgs, SupportedProvider};

use super::{DiscoverError, Provider};

#[derive(Debug, Clone, Deserialize)]
struct ListDropletsResponse {
    pub droplets: Vec<Droplet>,
}

#[derive(Debug, Clone, Deserialize)]
struct Droplet {
    pub id: usize,
    pub name: String,
    pub networks: Networks,
    pub region: DropletRegion,
}

#[derive(Debug, Clone, Deserialize)]
struct DropletRegion {
    pub slug: String,
}

#[derive(Debug, Clone, Deserialize)]
struct Networks {
    pub v4: Vec<Network>,
    pub v6: Vec<Network>,
}

#[derive(Debug, Clone, Deserialize)]
struct Network {
    pub ip_address: String,
    pub netmask: String,
    pub gateway: String,
    #[serde(rename = "type")]
    pub variant: String,
}

#[derive(Debug, Clone)]
pub struct DOProvider {
    region: Option<String>,
    tag_name: String,
    api_token: String,
}

impl TryFrom<ParsedArgs> for DOProvider {
    type Error = DiscoverError;

    fn try_from(args: ParsedArgs) -> Result<Self, Self::Error> {
        let mut tag_name = None;
        let mut region = None;
        let mut api_token = None;

        for (key, value) in args {
            match &key[..] {
                "tag_name" => tag_name = Some(value),
                "region" => region = Some(value),
                "api_token" => api_token = Some(value),
                "provider" => (),
                _ => return Err(DiscoverError::UnexpectedArgument(key)),
            }
        }

        let tag_name = tag_name.ok_or_else(|| DiscoverError::MissingArgument("tag_name".into()))?;
        let api_token = match api_token {
            Some(val) => val,
            None => env::var("API_TOKEN")
                .map_err(|_| DiscoverError::MissingArgument("api_token".into()))?,
        };

        Ok(DOProvider {
            tag_name,
            region,
            api_token,
        })
    }
}

impl TryFrom<Vec<String>> for DOProvider {
    type Error = DiscoverError;

    fn try_from(value: Vec<String>) -> Result<Self, Self::Error> {
        let args = ParsedArgs::try_from(value)?;
        match *args.provider() {
            SupportedProvider::DigitalOcean => DOProvider::try_from(args),
            _ => Err(DiscoverError::MalformedArgument(
                format!("provider={}", args.provider()),
                "you should not see this ...".to_string(),
            )),
        }
    }
}

impl DOProvider {
    pub fn tag_name(&self) -> &str {
        &self.tag_name
    }

    pub fn region(&self) -> Option<&String> {
        self.region.as_ref()
    }

    pub fn api_token(&self) -> &str {
        &self.api_token
    }

    async fn get_droplets(&self) -> Result<Vec<Droplet>, DiscoverError> {
        if let Some(region) = &self.region {
            debug!("Using region={} tag_name={}", region, self.tag_name);
        } else {
            debug!("Using tag_name={}", self.tag_name);
        }

        let mut droplets = Vec::new();

        let mut page = 1;
        let per_page = 200;

        loop {
            let res = reqwest::Client::new()
                .get(format!(
                    "https://api.digitalocean.com/v2/droplets?page={}&per_page={}&tag_name={}",
                    page, per_page, self.tag_name
                ))
                .header("Authorization", format!("Bearer {}", self.api_token))
                .send()
                .await;

            let data = res.map_err(|e| DiscoverError::ProviderRequestFailed(format!("{:?}", e)))?;
            let data = data
                .json::<ListDropletsResponse>()
                .await
                .map_err(|e| DiscoverError::ProviderRequestFailed(format!("{:?}", e)))?;
            let droplets_count = data.droplets.len();
            droplets.extend(data.droplets.into_iter());

            page += 1;

            if droplets_count < per_page {
                break;
            }
        }

        Ok(droplets)
    }
}

#[async_trait::async_trait]
impl Provider for DOProvider {
    async fn addrs(&self) -> Result<Vec<String>, DiscoverError> {
        let droplets = self.get_droplets().await?;
        debug!("Found {} droplets", droplets.len());

        let addrs = droplets
            .into_iter()
            .filter_map(|droplet| {
                let networks = droplet.networks;

                // Check region if specified
                if let Some(region) = &self.region {
                    if droplet.region.slug != *region {
                        return None;
                    }
                }

                let mut addrs = Vec::new();
                for ipv4 in &networks.v4 {
                    if ipv4.variant == "private" {
                        info!(
                            "Found instance {} ({}) with private IP: {}",
                            droplet.name, droplet.id, ipv4.ip_address
                        );

                        addrs.push(ipv4.ip_address.to_string());
                    }
                }
                // for ipv6 in &networks.v6 {
                // addrs.push(ipv6.ip_address.to_string());
                // }

                Some(addrs)
            })
            .flatten()
            .collect::<Vec<_>>();

        debug!("Found ip addresses: {:?}", addrs);
        Ok(addrs)
    }

    fn help() -> &'static str {
        "DigitalOcean:

	provider:  \"digitalocean\"
	region:    The DigitalOcean region to filter on
	tag_name:  The tag name to filter on
	api_token: The DigitalOcean API token to use
"
    }
}
