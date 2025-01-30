mod loadlibrary;
mod ipv4;
mod icmp;

fn main() {
    icmp::ping(ipv4::Addr([8, 8, 8, 8])).unwrap();
}
