//! Native DNS implementation using hickory-resolver.

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use hickory_resolver::{
    config::{ResolverConfig, ResolverOpts},
    name_server::TokioConnectionProvider,
    Resolver, TokioResolver,
};
use rhizome_pith_dns::Error;

/// Native DNS resolver.
pub struct NativeResolver {
    inner: TokioResolver,
}

impl NativeResolver {
    /// Create a new resolver using system configuration.
    pub fn new() -> Result<Self, Error> {
        let inner = Resolver::builder_tokio()
            .map_err(|e| Error::Lookup(e.to_string()))?
            .build();
        Ok(Self { inner })
    }

    /// Create a resolver using Google's public DNS.
    pub fn google() -> Self {
        let inner = Resolver::builder_with_config(
            ResolverConfig::google(),
            TokioConnectionProvider::default(),
        )
        .with_options(ResolverOpts::default())
        .build();
        Self { inner }
    }

    /// Create a resolver using Cloudflare's public DNS.
    pub fn cloudflare() -> Self {
        let inner = Resolver::builder_with_config(
            ResolverConfig::cloudflare(),
            TokioConnectionProvider::default(),
        )
        .with_options(ResolverOpts::default())
        .build();
        Self { inner }
    }
}

impl Default for NativeResolver {
    fn default() -> Self {
        Self::new().expect("failed to create resolver")
    }
}

impl rhizome_pith_dns::Resolver for NativeResolver {
    async fn lookup_ipv4(&self, host: &str) -> Result<Vec<Ipv4Addr>, Error> {
        let response = self
            .inner
            .ipv4_lookup(host)
            .await
            .map_err(|e| Error::Lookup(e.to_string()))?;
        let addrs: Vec<Ipv4Addr> = response.iter().map(|a| a.0).collect();
        if addrs.is_empty() {
            return Err(Error::NoRecords);
        }
        Ok(addrs)
    }

    async fn lookup_ipv6(&self, host: &str) -> Result<Vec<Ipv6Addr>, Error> {
        let response = self
            .inner
            .ipv6_lookup(host)
            .await
            .map_err(|e| Error::Lookup(e.to_string()))?;
        let addrs: Vec<Ipv6Addr> = response.iter().map(|a| a.0).collect();
        if addrs.is_empty() {
            return Err(Error::NoRecords);
        }
        Ok(addrs)
    }

    async fn lookup_ip(&self, host: &str) -> Result<Vec<IpAddr>, Error> {
        let response = self
            .inner
            .lookup_ip(host)
            .await
            .map_err(|e| Error::Lookup(e.to_string()))?;
        let addrs: Vec<IpAddr> = response.iter().collect();
        if addrs.is_empty() {
            return Err(Error::NoRecords);
        }
        Ok(addrs)
    }

    async fn lookup_txt(&self, host: &str) -> Result<Vec<String>, Error> {
        let response = self
            .inner
            .txt_lookup(host)
            .await
            .map_err(|e| Error::Lookup(e.to_string()))?;
        let records: Vec<String> = response
            .iter()
            .map(|txt| {
                txt.iter()
                    .map(|data| String::from_utf8_lossy(data).into_owned())
                    .collect::<Vec<_>>()
                    .join("")
            })
            .collect();
        if records.is_empty() {
            return Err(Error::NoRecords);
        }
        Ok(records)
    }

    async fn lookup_mx(&self, domain: &str) -> Result<Vec<(u16, String)>, Error> {
        let response = self
            .inner
            .mx_lookup(domain)
            .await
            .map_err(|e| Error::Lookup(e.to_string()))?;
        let records: Vec<(u16, String)> = response
            .iter()
            .map(|mx| (mx.preference(), mx.exchange().to_string()))
            .collect();
        if records.is_empty() {
            return Err(Error::NoRecords);
        }
        Ok(records)
    }

    async fn reverse_lookup(&self, addr: IpAddr) -> Result<Vec<String>, Error> {
        let response = self
            .inner
            .reverse_lookup(addr)
            .await
            .map_err(|e| Error::Lookup(e.to_string()))?;
        let names: Vec<String> = response.iter().map(|name| name.to_string()).collect();
        if names.is_empty() {
            return Err(Error::NoRecords);
        }
        Ok(names)
    }
}
