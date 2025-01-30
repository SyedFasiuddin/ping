mod icmp;
mod ipv4;
mod loadlibrary;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = std::env::args().nth(1).unwrap_or_else(|| {
        eprintln!("Usage: ping <DEST>");
        std::process::exit(1);
    });
    icmp::Request::new(addr.parse()?)
        .data("Foo Bar Baz")
        .send()?;

    Ok(())
}
