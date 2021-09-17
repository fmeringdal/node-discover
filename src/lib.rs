//! Retrieve IP addresses of nodes from a provider.
//!
//! Query DO for droplets in region "lon1" and with tag "cool-tag"
//! ```rust no_run
//! use node_discover::get_addresses;
//!
//! #[tokio::main]
//! async fn main() {
//!     let args = "provider=digitalocean region=lon1 tag_name=cool-tag".to_string();
//!     let res = get_addresses(args).await;
//!     match res {
//!         Ok(addrs) => println!("{:?}", addrs),
//!         Err(e) => println!("Error: {:?}", e),
//!     };
//! }
//! ```
//!
//! Query AWS for instances in region "eu-west-1" and with name=cool-name and get their private ipv4 addrs
//! ```rust no_run
//! use node_discover::get_addresses;
//!
//! #[tokio::main]
//! async fn main() {
//!     let args = "provider=aws region=eu-west-1 tag_key=Name tag_value=cool-name addr_type=private_v4".to_string();
//!     let res = get_addresses(args).await;
//!     match res {
//!         Ok(addrs) => println!("{:?}", addrs),
//!         Err(e) => println!("Error: {:?}", e),
//!     };
//! }
//! ```
mod args;
mod errors;
mod providers;

use std::convert::TryFrom;

use args::ParsedArgs;
pub use args::SupportedProvider;
use errors::DiscoverError;
use providers::aws::AWSProvider;
use providers::digitalocean::DOProvider;
pub use providers::*;

pub async fn get_addresses(args: String) -> Result<Vec<String>, DiscoverError> {
    let args = ParsedArgs::try_from(args)?;
    match *args.provider() {
        SupportedProvider::AWS => {
            let p = AWSProvider::try_from(args)?;
            p.addrs().await
        }
        SupportedProvider::DigitalOcean => {
            let p = DOProvider::try_from(args)?;
            p.addrs().await
        }
    }
}
