use log::info;
use node_discover::get_addresses;
use std::env;

#[tokio::test]
pub async fn aws_provider() {
    let region = env::var("AWS_REGION").unwrap_or_default();
    let access_key_id = env::var("AWS_ACCESS_KEY_ID").unwrap_or_default();
    let secret_access_key = env::var("AWS_SECRET_ACCESS_KEY").unwrap_or_default();

    if region.is_empty() || access_key_id.is_empty() || secret_access_key.is_empty() {
        info!("Skipping AWS provider test. AWS credentials or region missing");
        return;
    }

    let tag_key = "consul";
    let tag_value = "server";
    let args = format!(
        "provider=aws region={} tag_key={} tag_value={}",
        region, tag_key, tag_value
    );
    let args = args.split(" ").map(String::from).collect::<Vec<_>>();
    let res = get_addresses(args).await;
    assert!(res.is_ok());
    let addrs = res.unwrap();
    assert_eq!(addrs.len(), 2);
}
