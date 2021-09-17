terraform {
  required_providers {
    digitalocean = {
      source = "digitalocean/digitalocean"
      version = "~> 2.0"
    }
  }
}

provider "digitalocean" {
}

resource "digitalocean_tag" "test" {
  name = "${var.prefix}-test-tag"
}

resource "digitalocean_droplet" "test-01" {
  count              = 2
  image              = var.do_image
  name               = "${var.prefix}-01"
  region             = var.do_region
  size               = var.do_size
  private_networking = true
  tags               = ["${digitalocean_tag.test.id}"]
}

resource "digitalocean_droplet" "test-02" {
  image              = var.do_image
  name               = "${var.prefix}-02"
  region             = var.do_region
  size               = var.do_size
  private_networking = true
}
