pub mod aws;
pub mod digitalocean;

use std::convert::TryFrom;

use crate::{args::ParsedArgs, errors::DiscoverError};

#[async_trait::async_trait]
pub trait Provider: TryFrom<ParsedArgs> + Send + Sync {
    /// Retrieve IP addresses of nodes in this provider.
    async fn addrs(&self) -> Result<Vec<String>, DiscoverError>;
    /// Returns text explaining how to use this provider.
    ///
    /// That means which attributes are available and what the value of those
    /// attributes can be. Any other information that the user of this
    /// provider needs to know should also be explained.
    fn help() -> &'static str;
}
