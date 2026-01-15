//! DNS interfaces.

use std::future::Future;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

/// DNS errors.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("lookup failed: {0}")]
    Lookup(String),
    #[error("no records found")]
    NoRecords,
}

/// A DNS resolver.
pub trait Resolver {
    /// Lookup IPv4 addresses for a hostname.
    fn lookup_ipv4(&self, host: &str) -> impl Future<Output = Result<Vec<Ipv4Addr>, Error>>;

    /// Lookup IPv6 addresses for a hostname.
    fn lookup_ipv6(&self, host: &str) -> impl Future<Output = Result<Vec<Ipv6Addr>, Error>>;

    /// Lookup IP addresses (both v4 and v6) for a hostname.
    fn lookup_ip(&self, host: &str) -> impl Future<Output = Result<Vec<IpAddr>, Error>>;

    /// Lookup TXT records for a hostname.
    fn lookup_txt(&self, host: &str) -> impl Future<Output = Result<Vec<String>, Error>>;

    /// Lookup MX records for a domain. Returns (priority, exchange) pairs.
    fn lookup_mx(&self, domain: &str) -> impl Future<Output = Result<Vec<(u16, String)>, Error>>;

    /// Reverse lookup - get hostname for an IP address.
    fn reverse_lookup(&self, addr: IpAddr) -> impl Future<Output = Result<Vec<String>, Error>>;
}
