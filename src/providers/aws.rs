use log::{debug, info};
use rusoto_core::{HttpClient, Region};
use rusoto_credential::ChainProvider;
use rusoto_ec2::{DescribeInstancesRequest, DescribeInstancesResult, Ec2, Ec2Client, Filter};

use std::{convert::TryFrom, str::FromStr};

use crate::args::ParsedArgs;

use super::{DiscoverError, Provider};

#[derive(Debug, Clone, PartialEq)]
pub enum AddrType {
    PrivateV4,
    PublicV4,
    PublicV6,
}

// TODO: maybe just use serde instead
impl TryFrom<String> for AddrType {
    type Error = DiscoverError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match &value[..] {
            "public_v4" => Ok(AddrType::PublicV4),
            "public_v6" => Ok(AddrType::PublicV6),
            "private_v4" => Ok(AddrType::PrivateV4),
            _ => Err(DiscoverError::MalformedArgument(
                format!("addr_type={}", value),
                "Invalid addr_type".to_string(),
            )),
        }
    }
}

#[derive(Debug, Clone)]
pub struct AWSProvider {
    tag_key: String,
    tag_value: String,
    region: Region,
    addr_type: AddrType,
}

impl TryFrom<ParsedArgs> for AWSProvider {
    type Error = DiscoverError;

    fn try_from(args: ParsedArgs) -> Result<Self, Self::Error> {
        let mut tag_key = None;
        let mut tag_value = None;
        let mut region = None;
        let mut addr_type = AddrType::PrivateV4;

        for (key, value) in args {
            match &key[..] {
                "tag_key" => tag_key = Some(value),
                "tag_value" => tag_value = Some(value),
                "region" => {
                    region = Some(Region::from_str(&value).map_err(|_| {
                        DiscoverError::MalformedArgument(
                            format!("region={}", value),
                            "The value is not a valid AWS Region".to_string(),
                        )
                    })?)
                }
                "addr_type" => addr_type = AddrType::try_from(value)?,
                "provider" => (),
                _ => return Err(DiscoverError::UnexpectedArgument(key)),
            }
        }

        let tag_key = tag_key.ok_or_else(|| DiscoverError::MissingArgument("tag_key".into()))?;
        let tag_value =
            tag_value.ok_or_else(|| DiscoverError::MissingArgument("tag_value".into()))?;

        // https://rusoto.github.io/rusoto/rusoto_core/region/enum.Region.html
        let region = region.unwrap_or_default();

        Ok(AWSProvider {
            tag_key,
            tag_value,
            region,
            addr_type,
        })
    }
}

impl TryFrom<Vec<String>> for AWSProvider {
    type Error = DiscoverError;

    fn try_from(value: Vec<String>) -> Result<Self, Self::Error> {
        let args = ParsedArgs::try_from(value)?;
        match args.get("provider") {
            None => Err(DiscoverError::MissingArgument("provider".into())),
            Some(provider) => match &provider[..] {
                "aws" => AWSProvider::try_from(args),
                _ => Err(DiscoverError::MalformedArgument(
                    format!("provider={}", provider),
                    "you should not see this ...".to_string(),
                )),
            },
        }
    }
}

impl AWSProvider {
    pub fn tag_key(&self) -> &str {
        &self.tag_key
    }

    pub fn tag_value(&self) -> &str {
        &self.tag_value
    }

    pub fn region(&self) -> &Region {
        &self.region
    }

    pub fn addr_type(&self) -> &AddrType {
        &self.addr_type
    }

    async fn get_instances(&self) -> Result<DescribeInstancesResult, DiscoverError> {
        let provider = ChainProvider::new();
        let client = Ec2Client::new_with(HttpClient::new().unwrap(), provider, self.region.clone());

        let mut input = DescribeInstancesRequest::default();
        let mut filters: Vec<Filter> = Vec::new();
        filters.push(Filter {
            name: Some(format!("tag:{}", self.tag_key.clone())),
            values: Some(vec![self.tag_value.clone()]),
        });
        filters.push(Filter {
            name: Some("instance-state-name".into()),
            values: Some(vec!["running".into()]),
        });

        input.filters = Some(filters);

        debug!(
            "Using region={:?} tag_key={:?} tag_value={:?} addr_type={:?}",
            self.region, self.tag_key, self.tag_value, self.addr_type
        );

        client.describe_instances(input).await.map_err(|e| {
            println!("Provider req failed: {:?}", e);
            DiscoverError::ProviderRequestFailed(format!("{:?}", e))
        })
    }
}

#[async_trait::async_trait]
impl Provider for AWSProvider {
    async fn addrs(&self) -> Result<Vec<String>, DiscoverError> {
        let res = self.get_instances().await?;
        let reservations = res.reservations.unwrap_or_default();
        debug!("Found {} reservations", reservations.len());

        let addrs = reservations
            .into_iter()
            .map(|reservation| {
                let reservation_id = reservation.reservation_id.clone()
			.expect("Reservation to have a reservation id");
                let instances = reservation.instances.unwrap_or_default();
                debug!(
                    "Reservation {:?} has {} instances",
                    reservation_id,
                    instances.len()
                );
                instances
                    .into_iter()
                    .filter_map(|instance| {
                        let instance_id = instance.instance_id.clone()
				.expect("Instance to have an instance id");
                        debug!("Found instance {:?}", instance_id);

                        match self.addr_type {
                            AddrType::PrivateV4 => match instance.private_ip_address {
                                Some(addr) => {
                                    info!("Instance {:?} has private ip {:?}", instance_id, addr);

                                    Some(vec![addr])
                                }
                                None => {
                                    debug!("Instance {:?} has no private ip", instance_id);
                                    None
                                }
                            },
                            AddrType::PublicV4 => match instance.public_ip_address {
                                Some(addr) => {
                                    info!("Instance {:?} has public ip {:?}", instance_id, addr);

                                    Some(vec![addr])
                                }
                                None => {
                                    debug!("Instance {:?} has no public ip", instance_id);
                                    None
                                }
                            },
                            AddrType::PublicV6 => {
                                let network_interfaces =
                                    instance.network_interfaces.unwrap_or_default();
                                debug!(
                                    "Instance {:?} has {} network interfaces",
                                    instance_id,
                                    network_interfaces.len()
                                );

                                Some(
                                    network_interfaces
                                        .into_iter()
                                        .filter_map(|i| {
						let network_interface_id = i.network_interface_id.clone();
                                            debug!(
                                                "Checking NetworInterfaceId {:?} on Instance {:?}",
                                                network_interface_id, instance_id
                                            );

                                            if i.ipv_6_addresses.is_none() {
                                                debug!(
                                            "Instance {:?} has no IPv6 on NetworkInterfaceId {:?}",
                                            instance_id, network_interface_id
                                        );
                                            }

                                            i.ipv_6_addresses.map(|ipv6| {
                                                ipv6.into_iter()
                                                    .filter_map(|ipv6| {
                                                        info!("Instance {:?} has IPv6 {:?} on NetworkInterfaceId {:?}", instance_id, ipv6.ipv_6_address, network_interface_id);

                                                        ipv6.ipv_6_address
                                                    })
                                                    .collect::<Vec<_>>()
                                            })
                                        })
					.flatten()
                                        .collect::<Vec<_>>(),
                                )
                            }
                        }
                    })
                    .flatten()
                    .collect::<Vec<_>>()
            })
            .flatten()
            .collect::<Vec<_>>();

        Ok(addrs)
    }

    fn help() -> &'static str {
        "Amazon AWS:

	provider:          \"aws\"
	region:            The AWS region. Default to region of instance.
	tag_key:           The tag key to filter on
	tag_value:         The tag value to filter on
	addr_type:         \"private_v4\", \"public_v4\" or \"public_v6\". Defaults to \"private_v4\".
	access_key_id:     The AWS access key to use
	secret_access_key: The AWS secret access key to use

	The only required IAM permission is 'ec2:DescribeInstances'. If the Consul agent is
	running on AWS instance it is recommended you use an IAM role, otherwise it is
	recommended you make a dedicated IAM user and access key used only for auto-joining.
	"
    }
}
