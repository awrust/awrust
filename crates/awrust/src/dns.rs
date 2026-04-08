use std::net::{Ipv4Addr, SocketAddr};
use std::time::Duration;

use tokio::net::UdpSocket;

const MAX_PACKET: usize = 512;
const HEADER_LEN: usize = 12;
const TYPE_A: u16 = 1;
const TYPE_AAAA: u16 = 28;
const CLASS_IN: u16 = 1;
const TTL: u32 = 60;

const FLAG_QR: u16 = 1 << 15;
const FLAG_AA: u16 = 1 << 10;
const FLAG_RD: u16 = 1 << 8;
const FLAG_RA: u16 = 1 << 7;
const RCODE_REFUSED: u16 = 5;

pub struct DnsConfig {
    pub listen_addr: SocketAddr,
    pub resolve_ip: Ipv4Addr,
    pub base_domain: String,
    pub upstream: SocketAddr,
}

pub async fn serve(config: DnsConfig) {
    let socket = crate::net::bind_udp(config.listen_addr).await;
    tracing::info!(
        addr = %config.listen_addr,
        base_domain = %config.base_domain,
        resolve_ip = %config.resolve_ip,
        upstream = %config.upstream,
        "dns responder started"
    );

    let mut buf = [0u8; MAX_PACKET];
    loop {
        let (len, src) = match socket.recv_from(&mut buf).await {
            Ok(v) => v,
            Err(e) => {
                tracing::debug!(error = %e, "dns recv error");
                continue;
            }
        };

        let response = handle_query(&buf[..len], &config).await;
        if let Some(resp) = response {
            let _ = socket.send_to(&resp, src).await;
        }
    }
}

async fn handle_query(query: &[u8], config: &DnsConfig) -> Option<Vec<u8>> {
    if query.len() < HEADER_LEN {
        return None;
    }

    let flags = u16::from_be_bytes([query[2], query[3]]);
    if flags & FLAG_QR != 0 {
        return None;
    }

    let qdcount = u16::from_be_bytes([query[4], query[5]]);
    if qdcount != 1 {
        return None;
    }

    let (qname, qname_end) = parse_qname(query, HEADER_LEN)?;
    if qname_end + 4 > query.len() {
        return None;
    }

    let qtype = u16::from_be_bytes([query[qname_end], query[qname_end + 1]]);
    let qclass = u16::from_be_bytes([query[qname_end + 2], query[qname_end + 3]]);

    if qclass != CLASS_IN {
        return Some(build_refused(query));
    }

    let qname_lower = qname.to_ascii_lowercase();

    if !matches_base_domain(&qname_lower, &config.base_domain) {
        return forward_upstream(query, config.upstream).await;
    }

    match qtype {
        TYPE_A => Some(build_a_response(query, qname_end, config.resolve_ip)),
        TYPE_AAAA => Some(build_empty_response(query, qname_end)),
        _ => Some(build_empty_response(query, qname_end)),
    }
}

fn parse_qname(packet: &[u8], mut offset: usize) -> Option<(String, usize)> {
    let mut labels = Vec::new();

    loop {
        if offset >= packet.len() {
            return None;
        }
        let len = packet[offset] as usize;
        if len == 0 {
            offset += 1;
            break;
        }
        if len >= 64 {
            return None;
        }
        offset += 1;
        if offset + len > packet.len() {
            return None;
        }
        labels.push(std::str::from_utf8(&packet[offset..offset + len]).ok()?);
        offset += len;
    }

    Some((labels.join("."), offset))
}

fn matches_base_domain(qname: &str, base_domain: &str) -> bool {
    let base = base_domain.to_ascii_lowercase();
    if qname == base {
        return true;
    }
    qname.ends_with(&format!(".{base}"))
}

fn build_a_response(query: &[u8], question_end: usize, ip: Ipv4Addr) -> Vec<u8> {
    let question_section = &query[HEADER_LEN..question_end];
    let mut resp = Vec::with_capacity(HEADER_LEN + question_section.len() + 16);

    resp.extend_from_slice(&query[0..2]);

    let flags = FLAG_QR | FLAG_AA | FLAG_RD | FLAG_RA;
    resp.extend_from_slice(&flags.to_be_bytes());
    resp.extend_from_slice(&1u16.to_be_bytes());
    resp.extend_from_slice(&1u16.to_be_bytes());
    resp.extend_from_slice(&0u16.to_be_bytes());
    resp.extend_from_slice(&0u16.to_be_bytes());

    resp.extend_from_slice(question_section);

    resp.extend_from_slice(&[0xC0, 0x0C]);
    resp.extend_from_slice(&TYPE_A.to_be_bytes());
    resp.extend_from_slice(&CLASS_IN.to_be_bytes());
    resp.extend_from_slice(&TTL.to_be_bytes());
    resp.extend_from_slice(&4u16.to_be_bytes());
    resp.extend_from_slice(&ip.octets());

    resp
}

fn build_empty_response(query: &[u8], question_end: usize) -> Vec<u8> {
    let question_section = &query[HEADER_LEN..question_end];
    let mut resp = Vec::with_capacity(HEADER_LEN + question_section.len());

    resp.extend_from_slice(&query[0..2]);

    let flags = FLAG_QR | FLAG_AA | FLAG_RD | FLAG_RA;
    resp.extend_from_slice(&flags.to_be_bytes());
    resp.extend_from_slice(&1u16.to_be_bytes());
    resp.extend_from_slice(&0u16.to_be_bytes());
    resp.extend_from_slice(&0u16.to_be_bytes());
    resp.extend_from_slice(&0u16.to_be_bytes());

    resp.extend_from_slice(question_section);

    resp
}

fn build_refused(query: &[u8]) -> Vec<u8> {
    let mut resp = Vec::with_capacity(HEADER_LEN);
    resp.extend_from_slice(&query[0..2]);

    let flags = FLAG_QR | FLAG_RD | FLAG_RA | RCODE_REFUSED;
    resp.extend_from_slice(&flags.to_be_bytes());
    resp.extend_from_slice(&query[4..HEADER_LEN]);

    resp
}

async fn forward_upstream(query: &[u8], upstream: SocketAddr) -> Option<Vec<u8>> {
    let socket = UdpSocket::bind(crate::net::ephemeral_udp_addr(&upstream))
        .await
        .ok()?;
    socket.send_to(query, upstream).await.ok()?;
    let mut buf = [0u8; MAX_PACKET];
    let len = tokio::time::timeout(Duration::from_secs(3), socket.recv(&mut buf))
        .await
        .ok()?
        .ok()?;
    Some(buf[..len].to_vec())
}

pub fn detect_resolve_ip() -> Ipv4Addr {
    let socket = std::net::UdpSocket::bind("0.0.0.0:0").expect("bind ephemeral");
    socket.connect("8.8.8.8:53").expect("connect probe");
    match socket.local_addr().expect("local addr").ip() {
        std::net::IpAddr::V4(ip) => ip,
        _ => Ipv4Addr::LOCALHOST,
    }
}

pub fn detect_upstream() -> SocketAddr {
    if let Ok(contents) = std::fs::read_to_string("/etc/resolv.conf") {
        for line in contents.lines() {
            let line = line.trim();
            if let Some(addr_str) = line.strip_prefix("nameserver") {
                let addr_str = addr_str.trim();
                if let Ok(ip) = addr_str.parse::<std::net::IpAddr>() {
                    return SocketAddr::new(ip, 53);
                }
            }
        }
    }
    SocketAddr::new(std::net::IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8)), 53)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_query(id: u16, qname: &str, qtype: u16) -> Vec<u8> {
        let mut pkt = Vec::new();
        pkt.extend_from_slice(&id.to_be_bytes());
        pkt.extend_from_slice(&FLAG_RD.to_be_bytes());
        pkt.extend_from_slice(&1u16.to_be_bytes());
        pkt.extend_from_slice(&0u16.to_be_bytes());
        pkt.extend_from_slice(&0u16.to_be_bytes());
        pkt.extend_from_slice(&0u16.to_be_bytes());

        for label in qname.split('.') {
            pkt.push(label.len() as u8);
            pkt.extend_from_slice(label.as_bytes());
        }
        pkt.push(0);

        pkt.extend_from_slice(&qtype.to_be_bytes());
        pkt.extend_from_slice(&CLASS_IN.to_be_bytes());
        pkt
    }

    #[test]
    fn parse_qname_single_label() {
        let pkt = build_query(1, "localhost", TYPE_A);
        let (name, end) = parse_qname(&pkt, HEADER_LEN).unwrap();
        assert_eq!(name, "localhost");
        assert_eq!(pkt[end], 0x00);
    }

    #[test]
    fn parse_qname_multi_label() {
        let pkt = build_query(1, "mybucket.awrust.local", TYPE_A);
        let (name, _) = parse_qname(&pkt, HEADER_LEN).unwrap();
        assert_eq!(name, "mybucket.awrust.local");
    }

    #[test]
    fn parse_qname_rejects_truncated() {
        let pkt = [0u8; HEADER_LEN + 1];
        assert!(parse_qname(&pkt, HEADER_LEN + 1).is_none());
    }

    #[test]
    fn matches_exact_base_domain() {
        assert!(matches_base_domain("awrust", "awrust"));
    }

    #[test]
    fn matches_subdomain() {
        assert!(matches_base_domain("mybucket.awrust", "awrust"));
    }

    #[test]
    fn matches_deep_subdomain() {
        assert!(matches_base_domain("a.b.awrust", "awrust"));
    }

    #[test]
    fn rejects_unrelated_domain() {
        assert!(!matches_base_domain("google.com", "awrust"));
    }

    #[test]
    fn rejects_partial_suffix() {
        assert!(!matches_base_domain("notawrust", "awrust"));
    }

    #[test]
    fn matches_case_insensitive() {
        assert!(matches_base_domain("mybucket.awrust", "AWRUST"));
    }

    #[test]
    fn a_response_contains_ip() {
        let query = build_query(0xABCD, "mybucket.awrust", TYPE_A);
        let (_, qend) = parse_qname(&query, HEADER_LEN).unwrap();
        let qend = qend + 4;
        let resp = build_a_response(&query, qend, Ipv4Addr::new(172, 21, 0, 20));

        assert_eq!(resp[0], 0xAB);
        assert_eq!(resp[1], 0xCD);

        let flags = u16::from_be_bytes([resp[2], resp[3]]);
        assert_ne!(flags & FLAG_QR, 0);
        assert_ne!(flags & FLAG_AA, 0);

        let ancount = u16::from_be_bytes([resp[6], resp[7]]);
        assert_eq!(ancount, 1);

        let ip_offset = resp.len() - 4;
        assert_eq!(&resp[ip_offset..], &[172, 21, 0, 20]);
    }

    #[test]
    fn empty_response_has_no_answers() {
        let query = build_query(0x1234, "mybucket.awrust", TYPE_AAAA);
        let (_, qend) = parse_qname(&query, HEADER_LEN).unwrap();
        let qend = qend + 4;
        let resp = build_empty_response(&query, qend);

        let ancount = u16::from_be_bytes([resp[6], resp[7]]);
        assert_eq!(ancount, 0);

        let flags = u16::from_be_bytes([resp[2], resp[3]]);
        assert_ne!(flags & FLAG_QR, 0);
        assert_eq!(flags & 0x000F, 0);
    }

    #[test]
    fn refused_response_has_rcode_5() {
        let query = build_query(0x5678, "mybucket.awrust", TYPE_A);
        let resp = build_refused(&query);

        let rcode = u16::from_be_bytes([resp[2], resp[3]]) & 0x000F;
        assert_eq!(rcode, 5);
    }

    #[tokio::test]
    async fn handle_a_query_matching_domain() {
        let config = DnsConfig {
            listen_addr: "127.0.0.1:0".parse().unwrap(),
            resolve_ip: Ipv4Addr::new(10, 0, 0, 1),
            base_domain: "awrust".to_string(),
            upstream: "127.0.0.1:0".parse().unwrap(),
        };

        let query = build_query(0x0001, "mybucket.awrust", TYPE_A);
        let resp = handle_query(&query, &config).await.unwrap();

        let ancount = u16::from_be_bytes([resp[6], resp[7]]);
        assert_eq!(ancount, 1);

        let ip_offset = resp.len() - 4;
        assert_eq!(&resp[ip_offset..], &[10, 0, 0, 1]);
    }

    #[tokio::test]
    async fn handle_aaaa_query_matching_domain() {
        let config = DnsConfig {
            listen_addr: "127.0.0.1:0".parse().unwrap(),
            resolve_ip: Ipv4Addr::new(10, 0, 0, 1),
            base_domain: "awrust".to_string(),
            upstream: "127.0.0.1:0".parse().unwrap(),
        };

        let query = build_query(0x0002, "mybucket.awrust", TYPE_AAAA);
        let resp = handle_query(&query, &config).await.unwrap();

        let ancount = u16::from_be_bytes([resp[6], resp[7]]);
        assert_eq!(ancount, 0);
    }

    #[tokio::test]
    async fn handle_rejects_response_packets() {
        let config = DnsConfig {
            listen_addr: "127.0.0.1:0".parse().unwrap(),
            resolve_ip: Ipv4Addr::LOCALHOST,
            base_domain: "awrust".to_string(),
            upstream: "127.0.0.1:0".parse().unwrap(),
        };

        let mut query = build_query(0x0001, "test.awrust", TYPE_A);
        let flags = u16::from_be_bytes([query[2], query[3]]) | FLAG_QR;
        query[2..4].copy_from_slice(&flags.to_be_bytes());

        assert!(handle_query(&query, &config).await.is_none());
    }

    #[tokio::test]
    async fn handle_rejects_short_packets() {
        let config = DnsConfig {
            listen_addr: "127.0.0.1:0".parse().unwrap(),
            resolve_ip: Ipv4Addr::LOCALHOST,
            base_domain: "awrust".to_string(),
            upstream: "127.0.0.1:0".parse().unwrap(),
        };

        assert!(handle_query(&[0u8; 4], &config).await.is_none());
    }

    #[tokio::test]
    async fn roundtrip_via_udp() {
        let listener = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let dns_addr = listener.local_addr().unwrap();
        drop(listener);

        let config = DnsConfig {
            listen_addr: dns_addr,
            resolve_ip: Ipv4Addr::new(192, 168, 1, 100),
            base_domain: "test.local".to_string(),
            upstream: "127.0.0.1:0".parse().unwrap(),
        };

        tokio::spawn(serve(config));

        tokio::time::sleep(Duration::from_millis(50)).await;

        let client = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let query = build_query(0xBEEF, "mybucket.test.local", TYPE_A);
        client.send_to(&query, dns_addr).await.unwrap();

        let mut buf = [0u8; MAX_PACKET];
        let len = tokio::time::timeout(Duration::from_secs(2), client.recv(&mut buf))
            .await
            .unwrap()
            .unwrap();
        let resp = &buf[..len];

        assert_eq!(resp[0], 0xBE);
        assert_eq!(resp[1], 0xEF);

        let ancount = u16::from_be_bytes([resp[6], resp[7]]);
        assert_eq!(ancount, 1);

        let ip_offset = len - 4;
        assert_eq!(&resp[ip_offset..], &[192, 168, 1, 100]);
    }
}
