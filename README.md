# Rust Discover Nodes for Cloud Providers


`node-discover` is a Rust library and command line tool to discover
ip addresses of nodes in cloud environments based on meta information
like tags provided by the environment. It is a port to Rust of the excellent [go-discover](https://github.com/hashicorp/go-discover) library.

The configuration for the providers is provided as a list of `key=val key=val
...` tuples. 

Duplicate keys are reported as error and the provider is determined through the
`provider` key.

### Supported Providers

The following cloud providers have implementations in the node-discover/src/providers
package.

 * Amazon AWS [Config options](https://github.com/hashicorp/go-discover/blob/master/provider/aws/aws_discover.go#L19-L33)
 * DigitalOcean [Config options](https://github.com/hashicorp/go-discover/blob/master/provider/digitalocean/digitalocean_discover.go#L16-L24)

### Providers comming soon

 * Aliyun (Alibaba) Cloud [Config options](https://github.com/hashicorp/go-discover/blob/master/provider/aliyun/aliyun_discover.go#L15-L28)
 * Google Cloud [Config options](https://github.com/hashicorp/go-discover/blob/master/provider/gce/gce_discover.go#L17-L37)
 * Linode [Config options](https://github.com/hashicorp/go-discover/blob/master/provider/linode/linode_discover.go#L30-L41)
 * mDNS [Config options](https://github.com/hashicorp/go-discover/blob/master/provider/mdns/mdns_provider.go#L19-L31)
 * Microsoft Azure [Config options](https://github.com/hashicorp/go-discover/blob/master/provider/azure/azure_discover.go#L16-L37)
 * Openstack [Config options](https://github.com/hashicorp/go-discover/blob/master/provider/os/os_discover.go#L23-L38)
 * Scaleway [Config options](https://github.com/hashicorp/go-discover/blob/master/provider/scaleway/scaleway_discover.go#L14-L22)
 * SoftLayer [Config options](https://github.com/hashicorp/go-discover/blob/master/provider/softlayer/softlayer_discover.go#L16-L25)
 * TencentCloud [Config options](https://github.com/hashicorp/go-discover/blob/master/provider/tencentcloud/tencentcloud_discover.go#L23-L37)
 * Triton [Config options](https://github.com/hashicorp/go-discover/blob/master/provider/triton/triton_discover.go#L17-L27)
 * vSphere [Config options](https://github.com/hashicorp/go-discover/blob/master/provider/vsphere/vsphere_discover.go#L148-L155)
 * Packet [Config options](https://github.com/hashicorp/go-discover/blob/master/provider/packet/packet_discover.go#L25-L35)

### Config Example

```
# Aliyun (Alibaba) Cloud
provider=aliyun region=... tag_key=consul tag_value=... access_key_id=... access_key_secret=...

# Amazon AWS
provider=aws region=eu-west-1 tag_key=consul tag_value=... access_key_id=... secret_access_key=...

# DigitalOcean
provider=digitalocean region=... tag_name=... api_token=...

# Google Cloud
provider=gce project_name=... zone_pattern=eu-west-* tag_value=consul credentials_file=...

# Linode
provider=linode tag_name=... region=us-east address_type=private_v4 api_token=...

# mDNS
provider=mdns service=consul domain=local

# Microsoft Azure
provider=azure tag_name=consul tag_value=... tenant_id=... client_id=... subscription_id=... secret_access_key=...

# Openstack
provider=os tag_key=consul tag_value=server username=... password=... auth_url=...

# Scaleway
provider=scaleway organization=my-org tag_name=consul-server token=... region=...

# SoftLayer
provider=softlayer datacenter=dal06 tag_value=consul username=... api_key=...

# TencentCloud
provider=tencentcloud region=ap-guangzhou tag_key=consul tag_value=... access_key_id=... access_key_secret=...

# Triton
provider=triton account=testaccount url=https://us-sw-1.api.joyentcloud.com key_id=... tag_key=consul-role tag_value=server

# vSphere
provider=vsphere category_name=consul-role tag_name=consul-server host=... user=... password=... insecure_ssl=[true|false]

# Packet
provider=packet auth_token=token project=uuid url=... address_type=...

# Kubernetes
provider=k8s label_selector="app = consul-server"
```

## Command Line Tool Usage

Install the command line tool with:

```
cargo install node-discover
```

Then run it with:

```
$ node-discover help
$ node-discover help aws
$ node-discover addrs provider=aws region=eu-west-1 ...
```

## Library Usage

Active providers in Cargo.toml
```
[dependencies]
# Choose which providers you need in features list
node-discover = { version = "x.y.z", features = ["digitalocean", "aws"] }
```

For complete API documentation, see
[docs.rs](https://docs.rs/node-discover).
[crates.io](https://crates.io/crates/node-discover)

## Testing

Configuration tests can be run with:

```
$ cargo test
```

By default tests that communicate with providers do not run unless credentials
are set for that provider. To run provider tests you must set the necessary
environment variables.

**Note: This will make real API calls to the account provided by the credentials.**

```
$ AWS_ACCESS_KEY_ID=... AWS_ACCESS_KEY_SECRET=... AWS_REGION=... cargo test
```

This requires resources to exist that match those specified in tests
(eg instance tags in the case of AWS). To create these resources,
there are sets of [Terraform](https://www.terraform.io) configuration
in the `tests/tf` directory for supported providers.

You must use the same account and access credentials above. The same
environment variables should be applicable and read by Terraform.

```
$ cd tests/tf/aws
$ export AWS_ACCESS_KEY_ID=... AWS_ACCESS_KEY_SECRET=... AWS_REGION=...
$ terraform init
...
$ terraform apply
...
```

After Terraform successfully runs, you should be able to successfully
run the tests, assuming you have exported credentials into
your environment:

```
$ cargo test
```

To destroy the resources you need to use Terraform again:

```
$ cd tests/tf/aws
$ terraform destroy
...
```

**Note: There should be no requirements to create and test these resources other
than credentials and Terraform. This is to ensure tests can run in development
and CI environments consistently across all providers.**
