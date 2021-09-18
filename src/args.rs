use std::{
    collections::{hash_map::IntoIter, HashMap},
    convert::TryFrom,
    fmt::Display,
};

use crate::errors::DiscoverError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum SupportedProvider {
    #[serde(rename = "aws")]
    AWS,
    #[serde(rename = "digitalocean")]
    DigitalOcean,
}

impl Display for SupportedProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let provider = serde_json::to_string(self).expect("To serialize supported provider name");
        write!(f, "{}", provider)
    }
}

/// A utility type for parsing and working with the CLI arguments
#[derive(Debug, Clone)]
pub struct ParsedArgs {
    inner: HashMap<String, String>,
    provider: SupportedProvider,
}

impl ParsedArgs {
    pub fn get(&self, key: &str) -> Option<&String> {
        self.inner.get(key)
    }

    pub fn provider(&self) -> &SupportedProvider {
        &self.provider
    }
}

impl IntoIterator for ParsedArgs {
    type Item = (String, String);

    type IntoIter = IntoIter<String, String>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}

impl TryFrom<Vec<String>> for ParsedArgs {
    type Error = DiscoverError;

    fn try_from(value: Vec<String>) -> Result<Self, Self::Error> {
        let mut args = HashMap::with_capacity(value.len());
        for arg_str in value {
            let arg = arg_str.splitn(2, '=').collect::<Vec<_>>();

            if arg.len() != 2 || arg[1].is_empty() {
                return Err(DiscoverError::MalformedArgument(
                    arg[0].to_string(),
                    "Expected an argument on the format: key=value".to_string(),
                ));
            }

            // Fail on duplicate arg
            if args
                .insert(arg[0].to_string(), arg[1].to_string())
                .is_some()
            {
                return Err(DiscoverError::DuplicateArgument(arg[0].to_string()));
            }
        }

        let provider = match args.get("provider") {
            // provider must always be provided
            None => return Err(DiscoverError::MissingArgument("provider".into())),
            Some(p) => match &p.to_lowercase()[..] {
                "aws" => SupportedProvider::AWS,
                "digitalocean" => SupportedProvider::DigitalOcean,
                _ => return Err(DiscoverError::UnsupportedProvider(p.to_string())),
            },
        };

        Ok(Self {
            inner: args,
            provider,
        })
    }
}

impl TryFrom<String> for ParsedArgs {
    type Error = DiscoverError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let args = value
            .trim()
            .split(' ')
            .map(String::from)
            .collect::<Vec<_>>();
        ParsedArgs::try_from(args)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn fail_when_provider_is_not_provided() {
        let tag_key = "Name";
        let tag_value = "fsajfopja";
        let addr_type = "private_v4";

        let args = format!(
            "region=eu-west-1 tag_key={} tag_value={} addr_type={}",
            tag_key, tag_value, addr_type
        );

        let res = ParsedArgs::try_from(args);
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err(),
            DiscoverError::MissingArgument("provider".to_string())
        );
    }

    #[test]
    fn fail_on_duplicate_argument() {
        let inputs = vec!["provider=aws provider=do", "provider=aws provider=aws"];

        for input in inputs {
            let res = ParsedArgs::try_from(input.to_string());
            assert!(res.is_err());
            assert_eq!(
                res.unwrap_err(),
                DiscoverError::DuplicateArgument("provider".to_string())
            );
        }
    }

    #[test]
    fn fail_on_garbage_input() {
        let inputs = vec!["", "!!", "?"];

        for input in inputs {
            let res = ParsedArgs::try_from(input.to_string());
            assert!(res.is_err());
            assert_eq!(
                res.unwrap_err(),
                DiscoverError::MalformedArgument(
                    input.to_string(),
                    "Expected an argument on the format: key=value".to_string(),
                )
            );
        }
    }

    #[test]
    fn fail_on_malformed_args() {
        let malformed_args = vec!["=", "x:y", "zzzz", "t?x", "help=", "key"];

        for malformed_arg in malformed_args {
            let args = format!("provider=aws region=eu-west-1 {}", malformed_arg);
            let res = ParsedArgs::try_from(args);
            assert!(res.is_err());
            if malformed_arg.ends_with("=") {
                assert_eq!(
                    res.unwrap_err(),
                    DiscoverError::MalformedArgument(
                        malformed_arg[..malformed_arg.len() - 1].to_string(),
                        "Expected an argument on the format: key=value".to_string(),
                    )
                );
            } else {
                assert_eq!(
                    res.unwrap_err(),
                    DiscoverError::MalformedArgument(
                        malformed_arg.to_string(),
                        "Expected an argument on the format: key=value".to_string(),
                    )
                );
            }
        }
    }
}
