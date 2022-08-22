use std::{net::SocketAddr, time::Duration};

use anyhow::{anyhow, Result};
use http::header::{ACCEPT, CONTENT_TYPE};
use reqwest::{
    header::{HeaderMap, HeaderValue},
    Client,
};
use trust_dns_proto::{
    op::message::Message,
    rr::{Name, RData, RecordType},
};

use crate::utils::build_request_message;

pub struct BootstrapClient {
    https_client: Client,
}

impl BootstrapClient {
    pub fn new() -> Result<Self> {
        let mut headers = HeaderMap::new();
        headers.insert(
            CONTENT_TYPE,
            HeaderValue::from_str("application/dns-message").unwrap(),
        );
        headers.insert(
            ACCEPT,
            HeaderValue::from_str("application/dns-message").unwrap(),
        );

        let client_builder = Client::builder()
            .default_headers(headers)
            .https_only(true)
            .gzip(true)
            .brotli(true)
            .timeout(Duration::from_secs(10));

        Ok(BootstrapClient {
            https_client: client_builder.build()?,
        })
    }

    pub async fn bootstrap(
        &self,
        host: &str,
        bootstrap_upstream: Option<&str>,
    ) -> Result<SocketAddr> {
        let request_name = host.parse::<Name>()?;
        let request_message = build_request_message(request_name, RecordType::A);

        let raw_request_message = request_message.to_vec()?;

        let url = bootstrap_upstream.unwrap_or("https://1.0.0.1/dns-query");
        let request = self.https_client.post(url).body(raw_request_message);
        let response = request.send().await?;

        let raw_response_message = response.bytes().await?;

        let response_message = Message::from_vec(&raw_response_message)?;

        if response_message.answers().is_empty() {
            return Err(anyhow!("the response doesn't contain the answer"));
        }
        let record = &response_message.answers()[0];
        let record_data = record
            .data()
            .ok_or_else(|| anyhow!("the response doesn't contain the answer"))?;

        match record_data {
            RData::A(ipv4_address) => Ok(SocketAddr::new((*ipv4_address).into(), 0)),
            RData::AAAA(ipv6_address) => Ok(SocketAddr::new((*ipv6_address).into(), 0)),
            _ => Err(anyhow!("unknown record type")),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{
        collections::HashMap,
        net::{Ipv4Addr, SocketAddr},
    };

    use super::BootstrapClient;

    #[tokio::test]
    async fn test_bootstrap() {
        let bootstrap_client = BootstrapClient::new().unwrap();
        let bootstrap_result_map = HashMap::from([
            (
                "dns.google",
                vec![
                    SocketAddr::new(Ipv4Addr::new(8, 8, 8, 8).into(), 0),
                    SocketAddr::new(Ipv4Addr::new(8, 8, 4, 4).into(), 0),
                ],
            ),
            (
                "one.one.one.one",
                vec![
                    SocketAddr::new(Ipv4Addr::new(1, 1, 1, 1).into(), 0),
                    SocketAddr::new(Ipv4Addr::new(1, 0, 0, 1).into(), 0),
                ],
            ),
            (
                "dns.quad9.net",
                vec![
                    SocketAddr::new(Ipv4Addr::new(9, 9, 9, 9).into(), 0),
                    SocketAddr::new(Ipv4Addr::new(149, 112, 112, 112).into(), 0),
                ],
            ),
            (
                "dns.adguard.com",
                vec![
                    SocketAddr::new(Ipv4Addr::new(94, 140, 14, 14).into(), 0),
                    SocketAddr::new(Ipv4Addr::new(94, 140, 15, 15).into(), 0),
                ],
            ),
        ]);

        for (host, socket_addr_list) in bootstrap_result_map {
            let result = bootstrap_client.bootstrap(host, None).await.unwrap();
            assert!(socket_addr_list.contains(&result));
        }
    }
}
