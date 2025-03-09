use futures_util::{SinkExt, StreamExt};
use smoltcp::phy::ChecksumCapabilities;
use smoltcp::wire::{
    EthernetAddress, IpAddress, IpProtocol, IpVersion, Ipv4Address, Ipv4Cidr, Ipv4Packet, Ipv4Repr,
    TcpPacket, TcpRepr,
};
use tokio::net::TcpListener;
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::protocol::Message;

fn ingress_ipv4(package: &[u8]) {
    let mut checksum_caps = ChecksumCapabilities::default();

    let ipv4_packet = Ipv4Packet::new_unchecked(&package);
    let ipv4_repr =
        Ipv4Repr::parse(&ipv4_packet, &checksum_caps).expect("Failed to parse IPv4 packet");

    let src_addr = ipv4_repr.src_addr;
    let dst_addr = ipv4_repr.dst_addr;

    match ipv4_repr.next_header {
        IpProtocol::Tcp => {
            println!("TCP packet from {} to {}", src_addr, dst_addr);
            let tcp_packet = TcpPacket::new_unchecked(ipv4_packet.payload());
            let tcp_repr = TcpRepr::parse(
                &tcp_packet,
                &IpAddress::Ipv4(src_addr),
                &IpAddress::Ipv4(dst_addr),
                &checksum_caps,
            )
            .expect("Failed to parse TCP packet");
        }
        IpProtocol::Udp => {
            println!("UDP packet from {} to {}", src_addr, dst_addr);
        }
        _ => {
            println!("Other protocol packet from {} to {}", src_addr, dst_addr);
        }
    }
}

fn ingress_package(package: &[u8]) {
    match IpVersion::of_packet(&package) {
        Ok(IpVersion::Ipv4) => ingress_ipv4(package),
        Ok(IpVersion::Ipv6) => {
            // ignore
        }
        _ => {
            // error
        }
    }
}

pub async fn run_server(listen: &str, interface: &str, dmz_ports: &Vec<String>) {
    println!(
        "Running server on {} with interface {} and DMZ ports {:?}",
        listen, interface, dmz_ports
    );

    // find the ip and the gateway of the interface
    let interface = pnet::datalink::interfaces()
        .into_iter()
        .find(|iface| iface.name == interface)
        .expect("Interface not found");

    let ip = interface.ips.first().expect("No IP address found");

    // Bind the TCP listener
    let listener = TcpListener::bind(listen).await.expect("Failed to bind");

    println!("WebSocket server listening on: {}", listen);

    while let Ok((stream, _)) = listener.accept().await {
        let ip = ip.clone();
        tokio::spawn(async move {
            let ws_stream = accept_async(stream)
                .await
                .expect("Error during the websocket handshake");

            println!(
                "New WebSocket connection: {}",
                ws_stream.get_ref().peer_addr().unwrap()
            );

            let (mut write, mut read) = ws_stream.split();

            write
                .send(Message::Text(
                    format!("ip: {}, mask: {}", ip.ip(), ip.mask()).into(),
                ))
                .await
                .expect("Failed to send message");

            while let Some(message) = read.next().await {
                match message {
                    Ok(Message::Binary(msg)) => {
                        write
                            .send(Message::Binary(msg.clone()))
                            .await
                            .expect("Failed to send message");

                        let vec = msg.clone().to_vec();
                        let package = vec.as_slice();

                        ingress_package(&package);
                    }
                    Ok(Message::Text(msg)) => {
                        // do nothing as of now
                        println!("Received message: {}", msg);
                    }
                    Ok(Message::Ping(ping)) => {
                        write
                            .send(Message::Pong(ping))
                            .await
                            .expect("Failed to send pong");
                    }
                    Ok(Message::Pong(_)) => {
                        println!("Received pong");
                    }
                    Ok(Message::Close(_)) => {
                        println!("Connection closed");
                        break;
                    }
                    Ok(other) => {
                        println!("Received unknown message: {:?}", other);
                    }
                    Err(e) => {
                        println!("Error processing message: {}", e);
                        break;
                    }
                }
            }
        });
    }
}
