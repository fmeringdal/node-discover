use log::info;
use node_discover::get_addresses;
use std::env;

#[tokio::test]
pub async fn digitalocean_provider() {
    let api_token = env::var("DIGITALOCEAN_TOKEN").unwrap_or_default();

    if api_token.is_empty() {
        info!("Skipping DigitalOcean provider test. DigitalOcean credentials missing");
        return;
    }

    let region = "lon1";
    let tag_name = "node-discover-test-tag";
    let args = format!(
        "provider=digitalocean region={} tag_name={} api_token={}",
        region, tag_name, api_token
    );
    let args = args.split(" ").map(String::from).collect::<Vec<_>>();
    let res = get_addresses(args).await;
    assert!(res.is_ok());
    let addrs = res.unwrap();
    assert_eq!(addrs.len(), 2);
}
