use std::{net::SocketAddr, sync::Arc};

use anyhow::Result;
use tap::TapFallible;
use tokio::net::UdpSocket;
use tracing::{error, info, instrument, warn};
use trust_dns_proto::op::Message;

use crate::upstream::HttpsClient;

#[derive(Debug)]
pub struct UdpListener {
    udp_socket: Arc<UdpSocket>,
    https_client: HttpsClient,
}

impl UdpListener {
    pub async fn new(listen_addr: SocketAddr, https_client: HttpsClient) -> Result<Self> {
        let udp_socket = UdpSocket::bind(listen_addr).await?;

        info!(%listen_addr, "udp socket listened");

        Ok(UdpListener {
            udp_socket: Arc::new(udp_socket),
            https_client,
        })
    }

    pub async fn listen(&self) {
        let mut buffer = [0; 4096];

        loop {
            let (_, addr) = match self.udp_socket.recv_from(&mut buffer).await {
                Ok(udp_recv_from_result) => udp_recv_from_result,
                Err(_) => {
                    warn!("failed to receive the datagram message");

                    continue;
                }
            };

            let request_message = match Message::from_vec(&buffer) {
                Ok(request_message) => request_message,
                Err(_) => {
                    warn!("failed to parse the request");

                    continue;
                }
            };

            let udp_socket = self.udp_socket.clone();
            let https_client = self.https_client.clone();

            tokio::spawn(
                async move { reply(request_message, &https_client, &udp_socket, addr).await },
            );
        }
    }
}

#[instrument(skip(https_client, udp_socket), err)]
async fn reply(
    request_message: Message,
    https_client: &HttpsClient,
    udp_socket: &UdpSocket,
    addr: SocketAddr,
) -> Result<()> {
    for request_record in request_message.queries().iter() {
        info!(
            phase = "request",
            "{} {} {}",
            request_record.name(),
            request_record.query_class(),
            request_record.query_type(),
        );
    }

    let response_message = https_client
        .process(request_message)
        .await
        .tap_err(|err| error!(%err, "process dns request failed"))?;

    for response_record in response_message.answers().iter() {
        info!(phase = "response", "{}", response_record);
    }

    let raw_response_message = response_message
        .to_vec()
        .tap_err(|err| error!(%err, "failed to marshal dns response"))?;

    udp_socket
        .send_to(&raw_response_message, &addr)
        .await
        .tap_err(|err| error!(%err, "failed to send dns response"))?;

    Ok(())
}
