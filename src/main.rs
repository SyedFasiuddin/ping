mod icmp;
mod ipv4;
mod loadlibrary;

fn main() {
    let addr = std::env::args().nth(1).unwrap_or_else(|| {
        eprintln!("Usage: ping <DEST>");
        std::process::exit(1);
    });
    icmp::ping(addr.parse().unwrap()).expect("ping failed");
}
