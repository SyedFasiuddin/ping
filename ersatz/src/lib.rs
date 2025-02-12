pub mod arp;
pub mod blob;
pub mod error;
pub mod ethernet;
pub mod icmp;
pub mod ipv4;
pub mod loadlibrary;
pub mod netinfo;
pub mod parse;
pub mod serialize;
pub mod vls;

use once_cell::sync::Lazy;
use std::sync::{mpsc, Arc, Mutex};
use std::time::Duration;

static RAWSOCK_LIB: Lazy<Box<dyn rawsock::traits::Library>> =
    Lazy::new(|| rawsock::open_best_library().unwrap());

pub struct Interface {
    nic: netinfo::NIC,
    gateway_mac: ethernet::Addr,
    iface: Arc<dyn rawsock::traits::DynamicInterface<'static>>,
    pending: Arc<Mutex<PendingQueries>>,
}

struct PendingQueries {
    ipv4: Vec<Box<dyn Fn(&ipv4::Packet) -> bool + Send + 'static>>,
}

impl Interface {
    pub fn open_default() -> Result<Self, error::Error> {
        let nic = netinfo::default_nic()?;
        let iface_name = format!(r#"\Device\NPF_{}"#, nic.guid);
        let iface = RAWSOCK_LIB.open_interface_arc(&iface_name)?;

        let pending = Arc::new(Mutex::new(PendingQueries { ipv4: Vec::new() }));

        let gateway_mac = crossbeam_utils::thread::scope(|s| {
            let (tx, rx) = mpsc::channel();
            let gateway_ip = nic.gateway;

            let poll_iface = iface.clone();
            s.spawn(move |_| {
                poll_iface
                    .loop_infinite_dyn(&mut |packet| {
                        let frame = match ethernet::Frame::parse(packet) {
                            Ok((_remaining, frame)) => frame,
                            _ => return,
                        };
                        let arp = match frame.payload {
                            ethernet::Payload::ARP(x) => x,
                            _ => return,
                        };

                        if let arp::Operation::Reply = arp.operation {
                            if arp.sender_ip_addr == gateway_ip {
                                tx.send(arp.sender_hw_addr).unwrap();
                            }
                        }
                    })
                    .unwrap();
            });

            arp::Packet::request(&nic, gateway_ip)
                .as_ethernet_payload()
                .as_broadcast_frame(&nic)
                .send(iface.as_ref());

            let ret = rx
                .recv_timeout(Duration::from_secs(3))
                .map_err(|_| "ARP timeout")
                .unwrap();
            iface.break_loop();
            ret
        })
        .unwrap();

        let res = Self {
            pending: pending.clone(),
            nic,
            gateway_mac,
            iface: iface.clone(),
        };

        std::thread::spawn(move || {
            iface
                .loop_infinite_dyn(&mut |packet| {
                    let frame = match ethernet::Frame::parse(packet) {
                        Ok((_, frame)) => frame,
                        _ => return,
                    };

                    if let ethernet::Payload::IPv4(ref packet) = frame.payload {
                        let mut pending = pending.lock().unwrap();
                        pending
                            .ipv4
                            .iter()
                            .position(|f| f(packet))
                            .map(|i| pending.ipv4.remove(i));
                    }
                })
                .unwrap();
        });

        Ok(res)
    }

    pub fn send_ipv4(&self, payload: ipv4::Payload, addr: &ipv4::Addr) -> Result<(), error::Error> {
        payload
            .as_packet(self.nic.address, addr.clone())
            .as_ethernet_payload()
            .as_frame(&self.nic, self.gateway_mac)
            .send(self.iface.as_ref());
        self.iface.as_ref().flush();
        Ok(())
    }

    pub fn expect_ipv4<F, T>(&self, f: F) -> mpsc::Receiver<T>
    where
        F: Fn(&ipv4::Packet) -> Option<T> + Send + 'static,
        T: Send + 'static,
    {
        let (tx, rx) = mpsc::channel();

        let mut pending = self.pending.lock().unwrap();
        pending.ipv4.push(Box::new(move |packet| {
            match f(&packet) {
                Some(val) => {
                    tx.send(val).unwrap_or(()); // ignore send errors
                    true
                }
                None => false,
            }
        }));
        rx
    }
}
