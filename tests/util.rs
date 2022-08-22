use https_dns::local::UdpListener;
use https_dns::upstream::HttpsClient;

pub async fn build_test_listener() -> UdpListener {
    let upstream_address = "https://cloudflare-dns.com:443";
    let local_address = "127.0.0.1:10053".parse().unwrap();

    let https_client = HttpsClient::new(upstream_address, None).await.unwrap();

    UdpListener::new(local_address, https_client).await.unwrap()
}
