use groot2_client::Groot2Client;
use std::time::Duration;

fn main() {
    let mut c = Groot2Client::local(1667);
    c.set_timeout(Duration::from_secs(2));
    let xml = c.request_full_tree().expect("fulltree");
    println!("--- first 1500 chars ---\n{}", &xml[..xml.len().min(1500)]);
    println!("--- total {} bytes ---", xml.len());

    // Try parsing it with btparse:
    match btparse::parse_xml(&xml) {
        Ok(trees) => {
            println!("btparse OK: {} trees", trees.len());
            for t in &trees {
                println!("  tree id={} root={}", t.id, t.root.registration_name);
            }
        }
        Err(e) => println!("btparse error: {e}"),
    }
}
