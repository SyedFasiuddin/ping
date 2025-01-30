mod icmp;
mod ipv4;
mod loadlibrary;

fn main() {
    let addr = std::env::args().nth(1).unwrap_or_else(|| {
        eprintln!("Usage: ping <DEST>");
        std::process::exit(1);
    });
    let dest = ipv4::Addr::parse(&addr).unwrap();
    icmp::ping(dest).expect("ping failed");
}
