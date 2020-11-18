use quick_xml::{events::Event, Reader};
use virt::{connect::Connect, error::Error};

fn poweron(dom: &virt::domain::Domain) -> bool {
    let name = dom.get_name().unwrap();
    match dom.is_active() {
        Ok(true) => {
            println!("Domain already started: {}", name);
            true
        }
        Ok(false) => match dom.create() {
            Ok(_) => {
                println!("Domain started: {}", name);
                true
            }
            Err(e) => {
                println!("Domain failed to start: {} ({})", name, e);
                false
            }
        },
        Err(e) => {
            println!("Failed to get status: {} ({})", name, e);
            false
        }
    }
}

fn search_mac(dom: &virt::domain::Domain, mac: &str) -> Result<bool, Error> {
    let mut buf: Vec<u8> = Vec::new();
    let mut buf2: Vec<u8> = Vec::new();

    let xml_desc = dom.get_xml_desc(0)?;

    let mut reader = Reader::from_str(&xml_desc);
    reader.trim_text(true);

    loop {
        match reader.read_event(&mut buf) {
            Ok(Event::Start(ref e)) => match e.name() {
                b"domain" | b"devices" | b"interface" => (),
                _ => reader.read_to_end(e.name(), &mut buf2).unwrap(),
            },
            Ok(Event::Empty(ref e)) => match e.name() {
                b"mac" => {
                    let mut mac_iter = e
                        .attributes()
                        .filter_map(Result::ok)
                        .map(|a| a.unescape_and_decode_value(&reader))
                        .filter_map(Result::ok);
                    if mac_iter.any(|m| m == mac) {
                        return Ok(true);
                    }
                    break;
                }
                _ => (),
            },
            Ok(Event::Eof) => break,
            Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
            _ => (),
        }
    }
    Ok(false)
}

pub fn search_and_boot(conn: &Connect, mac: &str) -> Result<bool, Error> {
    let flags = virt::connect::VIR_CONNECT_LIST_DOMAINS_ACTIVE
        | virt::connect::VIR_CONNECT_LIST_DOMAINS_INACTIVE;

    let doms = conn.list_all_domains(flags)?;

    for d in doms {
        match search_mac(&d, mac) {
            Ok(true) => return Ok(poweron(&d)),
            Ok(false) => continue,
            _ => (),
        }
    }
    println!("No matching domain found");
    Ok(false)
}
