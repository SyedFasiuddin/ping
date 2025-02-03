mod icmp;
mod ipv4;
mod loadlibrary;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = std::env::args().nth(1).unwrap_or_else(|| {
        eprintln!("Usage: ping <DEST>");
        std::process::exit(1);
    });

    let dest_addr = addr.parse()?;
    let data = "Hello World";

    println!("Pinging {:?} with {} bytes of data:", dest_addr, data.len());
    for _ in 0..4 {
        match icmp::Request::new(dest_addr)
            .ttl(128)
            .timeout(4000)
            .data(data)
            .send()
        {
            Ok(res) => println!(
                "Reply from {:?}: bytes={} time={:?} TTL={}",
                res.addr,
                res.data.len(),
                res.rtt,
                res.ttl
            ),
            Err(_) => println!("Something went wrong"),
        }

        std::thread::sleep(std::time::Duration::from_secs(1));
    }

    Ok(())
}
