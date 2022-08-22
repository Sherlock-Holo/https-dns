use std::{net::IpAddr, time::Duration};

use anyhow::{anyhow, Result};
use reqwest::{
    header::{HeaderMap, HeaderValue, CONTENT_TYPE},
    Client, Url,
};
use tracing::info;
use trust_dns_proto::op::message::Message;

use crate::bootstrap::BootstrapClient;
use crate::cache::Cache;

#[derive(Clone, Debug)]
pub struct HttpsClient {
    upstream: Url,
    https_client: Client,
    cache: Cache,
}

impl HttpsClient {
    pub async fn new(upstream: &str, bootstrap_upstream: Option<&str>) -> Result<Self> {
        let upstream = Url::parse(upstream)?;
        let host = upstream.host_str().ok_or_else(|| anyhow!("host is miss"))?;

        let mut headers = HeaderMap::new();
        headers.insert(
            CONTENT_TYPE,
            HeaderValue::from_str("application/dns-message").unwrap(),
        );

        let mut client_builder = Client::builder()
            .default_headers(headers)
            .https_only(true)
            .gzip(true)
            .brotli(true)
            .timeout(Duration::from_secs(10));

        if host.parse::<IpAddr>().is_err() {
            let bootstrap_client = BootstrapClient::new()?;
            let ip_addr = bootstrap_client.bootstrap(host, bootstrap_upstream).await?;

            client_builder = client_builder.resolve(host, ip_addr);
        }

        let https_client = client_builder.build()?;

        info!(%upstream, "connected to upstream");

        Ok(HttpsClient {
            upstream,
            https_client,
            cache: Cache::new(),
        })
    }

    pub async fn process(&self, request_message: Message) -> Result<Message> {
        if let Some(response_message) = self.cache.get(&request_message) {
            return Ok(response_message);
        }

        let raw_request_message = request_message.to_vec()?;

        let request = self
            .https_client
            .post(self.upstream.as_str())
            .body(raw_request_message);
        let response = request.send().await?;
        let raw_response_message = response.bytes().await?;

        let message = Message::from_vec(&raw_response_message)?;

        self.cache.put(message.clone());

        Ok(message)
    }
}
