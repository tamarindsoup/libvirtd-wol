use clap::Clap;
use eui48::MacAddress;
use pcap::Capture;
use virt::connect::Connect;

use libvirtd_wol::doms::search_and_boot;

#[derive(Clap)]
#[clap(version = "0.1.0", author = "tamarindsoup")]
struct Opts {
    #[clap(short = "i", long = "interface", default_value = "any")]
    interface: String,

    #[clap(short = "u", long = "uri", default_value = "qemu:///system")]
    uri: String,
}

fn main() {
    let opts: Opts = Opts::parse();

    let interface: &str = &opts.interface;
    println!("Listening on interface: {}", interface);
    let mut cap = Capture::from_device(interface).unwrap().open().unwrap();

    cap.filter("( ether proto 0x0842 or ( udp port 7 or 9 ) ) and greater 102")
        .unwrap();

    let mut conn = Connect::open(&opts.uri).expect("No connection to hypervisor");

    'outer: while let Ok(packet) = cap.next() {
        let mut iter = packet[packet.len() - 102..].chunks(6);
        if Some(&[255u8, 255u8, 255u8, 255u8, 255u8, 255u8][..]) != iter.next() {
            continue 'outer;
        }
        let raw_mac = iter.next().unwrap();
        while let Some(m) = iter.next() {
            if m != raw_mac {
                continue 'outer;
            }
        }
        let macaddr = MacAddress::from_bytes(raw_mac).unwrap().to_hex_string();
        println!("\nGot wol packet! {:?}", macaddr);
        search_and_boot(&conn, &macaddr).unwrap();
    }

    conn.close().expect("Failed to disconnect from hypervisor");
}
