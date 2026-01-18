// Docker support module
// Main implementation is in commands/docker.rs
// This module provides additional Docker utilities

use anyhow::Result;

pub const DOCKER_IMAGE: &str = "ghcr.io/pyrax-chain/inferno-node";
pub const CONTAINER_PREFIX: &str = "inferno-node";

pub fn get_image_name(tag: &str) -> String {
    format!("{}:{}", DOCKER_IMAGE, tag)
}

pub fn get_container_name(instance: u32) -> String {
    format!("{}-{}", CONTAINER_PREFIX, instance)
}
