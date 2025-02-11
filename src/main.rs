use std::time::Instant;
use std::time::Duration;

use ersatz::Interface;
use ersatz::ipv4;
use ersatz::icmp;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = std::env::args().nth(1).unwrap_or_else(|| {
        eprintln!("Usage: ping <DEST>");
        std::process::exit(1);
    });

    let dest = addr.parse()?;
    let data = "Hello World";
    let identifier = 0xC0DE;
    let iface = Interface::open_default()?;

    println!("Pinging {:?} with {} bytes of data:", dest, data.len());
    for sequence_number in 0..4 {
        let echo_request = ersatz::icmp::Echo {
            identifier,
            sequence_number,
        }
        .as_echo_request(data)
        .as_ipv4_payload();

        let before = Instant::now();
        let rx = iface.expect_ipv4(move |packet| {
            if let ipv4::Payload::ICMP(ref icmp_packet) = packet.payload {
                if let icmp::Header::EchoReply(ref reply) = icmp_packet.header {
                    if reply.identifier == identifier && reply.sequence_number == sequence_number {
                        return Some((before.elapsed(), packet.clone()));
                    }
                }
            }

            None
        });

        iface.send_ipv4(echo_request, &dest)?;
        match rx.recv_timeout(Duration::from_secs(3)) {
            Ok((elapsed, packet)) => {
                if let ipv4::Payload::ICMP(ref icmp_packet) = packet.payload {
                    if let icmp::Header::EchoReply(_) = icmp_packet.header {
                        println!(
                            "Reply from {:?}: bytes={} time={:?} TTL={}",
                            packet.src,
                            icmp_packet.payload.0.len(),
                            elapsed,
                            packet.ttl,
                        );
                    }
                }
            }
            Err(_) => {
                println!("Timed out!");
                std::process::exit(1);
            }
        }
        std::thread::sleep(std::time::Duration::from_secs(1));
    }

    Ok(())
}
