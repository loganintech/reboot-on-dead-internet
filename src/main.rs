use std::net::{IpAddr, Ipv4Addr};
use std::process::Command;
use std::thread::sleep;
use std::time::Duration;
use surge_ping::{IcmpPacket, SurgeError};
use futures::{executor, future};

const IPS_TO_PING: [IpAddr; 3] = [
    IpAddr::V4(Ipv4Addr::new(192, 168, 1, 10)),
    IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8)),
    IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1)),
];

#[tokio::main]
async fn main() {
    let mut times_in_a_row = 0;
    loop {
        let mut pings = vec![];
        for ip in IPS_TO_PING {
            pings.push(surge_ping::ping(ip, &[]));
        }
        let results = future::join_all(pings).await;
        let mut network_down_errors = 0;
        for res in results {
            match res {
                Ok((IcmpPacket::V4(packet), duration)) => {
                    println!(
                        "{} bytes from {}: icmp_seq={} ttl={:?} time={:.2?}",
                        packet.get_size(),
                        packet.get_source(),
                        packet.get_sequence(),
                        packet.get_ttl(),
                        duration
                    );
                }
                Ok(_) => { network_down_errors += 1; }
                Err(SurgeError::Timeout { seq }) => { network_down_errors += 1; }
                Err(SurgeError::NetworkError) => { network_down_errors += 1; }
                Err(e) => println!("{:?}", e),
            }
        }

        if network_down_errors == IPS_TO_PING.len() {
            // Add one to the times this has happened in a row
            times_in_a_row += 1;

            // If we're down for a full minute (6 times * 10 seconds sleep timer), then reboot
            if times_in_a_row >= 6 {
                // Reboots the computer
                if let Err(e) = system_shutdown::force_reboot() {
                    eprintln!("Error shutting down PC: {:?}", e);
                    Command::new("reboot").args(&["-r", "now"]).spawn().unwrap();
                };
            }
        } else {
            // If network is up, clear the consecutive failed count
            times_in_a_row = 0;
        }

        tokio::time::sleep(Duration::from_secs(10)).await;
    }
}
