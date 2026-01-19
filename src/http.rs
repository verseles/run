// Copyright (C) 2025 Verseles
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published
// by the Free Software Foundation, version 3 of the License.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Affero General Public License for more details.

//! HTTP client with custom DNS resolver for Termux compatibility.
//!
//! Uses Cloudflare's 1.1.1.1 DNS resolver to avoid issues with
//! broken system DNS in environments like Termux.

use hickory_resolver::{
    config::{NameServerConfig, ResolverConfig, ResolverOpts},
    name_server::TokioConnectionProvider,
    proto::xfer::Protocol,
    Resolver, TokioResolver,
};
use reqwest::dns::{Addrs, Name, Resolve, Resolving};
use std::io;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;

/// Custom DNS resolver using Cloudflare's 1.1.1.1
#[derive(Clone)]
pub struct HickoryDnsResolver {
    resolver: Arc<TokioResolver>,
}

impl HickoryDnsResolver {
    /// Create a new resolver using Cloudflare DNS (1.1.1.1)
    pub fn new() -> Self {
        // Configure Cloudflare DNS (1.1.1.1)
        let cloudflare_ip = IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1));
        let cloudflare_addr = SocketAddr::new(cloudflare_ip, 53);

        let name_server = NameServerConfig::new(cloudflare_addr, Protocol::Udp);
        let config = ResolverConfig::from_parts(None, vec![], vec![name_server]);

        let mut opts = ResolverOpts::default();
        opts.timeout = std::time::Duration::from_secs(5);
        opts.attempts = 2;

        // Use builder_with_config to create the resolver
        let resolver = Resolver::builder_with_config(config, TokioConnectionProvider::default())
            .with_options(opts)
            .build();

        Self {
            resolver: Arc::new(resolver),
        }
    }
}

impl Default for HickoryDnsResolver {
    fn default() -> Self {
        Self::new()
    }
}

impl Resolve for HickoryDnsResolver {
    fn resolve(&self, name: Name) -> Resolving {
        let resolver = self.resolver.clone();
        Box::pin(async move {
            let lookup = resolver
                .lookup_ip(name.as_str())
                .await
                .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { Box::new(e) })?;

            let addrs: Vec<SocketAddr> = lookup.iter().map(|ip| SocketAddr::new(ip, 0)).collect();

            if addrs.is_empty() {
                return Err(Box::new(io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("No addresses found for {}", name.as_str()),
                ))
                    as Box<dyn std::error::Error + Send + Sync>);
            }

            let addrs: Addrs = Box::new(addrs.into_iter());
            Ok(addrs)
        })
    }
}

/// Create a reqwest ClientBuilder with our custom DNS resolver.
///
/// This ensures compatibility with environments like Termux where
/// system DNS resolution may be unreliable.
pub fn create_client_builder() -> reqwest::ClientBuilder {
    let resolver = HickoryDnsResolver::new();
    reqwest::Client::builder().dns_resolver(Arc::new(resolver))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolver_creation() {
        let resolver = HickoryDnsResolver::new();
        assert!(Arc::strong_count(&resolver.resolver) == 1);
    }

    #[test]
    fn test_client_builder_creation() {
        let builder = create_client_builder();
        // Just verify it doesn't panic
        let _client = builder.build();
    }
}
