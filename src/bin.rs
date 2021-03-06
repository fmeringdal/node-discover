use log::error;
use node_discover::get_addresses;
#[cfg(any(feature = "aws", feature = "digitalocean"))]
use node_discover::Provider;

const GLOBAL_HELP: &str = "The options for discovering ip addresses are provided as a
single string value in \"key=value key=value ...\" format where
the values are URL encoded.

  provider=aws region=eu-west-1 ...

The options are provider specific and are listed below.
";

pub fn help(provider: &str) {
    println!("{}", GLOBAL_HELP);
    match provider {
        "aws" => {
            // Only print AWS help if it is enabled
            #[cfg(feature = "aws")]
            {
                return println!("{}", node_discover::AWSProvider::help());
            }
        }
        "digitalocean" => {
            // Only print DO help if it is enabled
            #[cfg(feature = "digitalocean")]
            {
                return println!("{}", node_discover::DOProvider::help());
            }
        }
        _ => {
            help("aws");
            help("digitalocean");
        }
    }
}

async fn get_addrs(args: Vec<String>) {
    let res = get_addresses(args.join(" ")).await;

    match res {
        Ok(addrs) => println!("{:?}", addrs),
        Err(e) => {
            error!("Unable to retrieve addresses. Received error: {}", e);
        }
    }
}

fn get_help(args: Vec<String>) {
    let provider = args.get(0);
    match provider {
        Some(provider) => match &provider[..] {
            "aws" => {
                help("aws");
            }
            "digitalocean" => {
                help("digitalocean");
            }
            _ => {
                help("all");
            }
        },
        None => {
            help("all");
        }
    };
}

#[tokio::main]
async fn main() {
    env_logger::init();

    // Collect args and skip the program name
    let mut args: Vec<_> = std::env::args().skip(1).collect();

    if args.is_empty() {
        help("all");
        return;
    }

    let cmd = args.remove(0);

    match &cmd[..] {
        "addrs" => get_addrs(args).await,
        _ => get_help(args),
    }
}
