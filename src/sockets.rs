// It is at this point, that I gave up making my code look nice and maintainable

use crate::{cli, constants};
use chrono::{DateTime, Duration, Utc};
use serde_json::json;
use std::net::ToSocketAddrs;
use std::sync::Arc;
use std::time::Instant;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::sleep;
use tokio_rustls::{rustls::ClientConfig, webpki::DNSNameRef, TlsConnector};

pub async fn auto_offset_calculation_regular(username_to_snipe: &str) -> i32 {
    println!("Measuring offset...");
    let mut buf = [0; 12];
    let addr = "api.minecraftservices.com:443"
        .to_socket_addrs()
        .unwrap()
        .next()
        .ok_or("failed to resolve api.minecraftservices.com")
        .unwrap();
    let data = format!("PUT /minecraft/profile/name/{} HTTP/1.1\r\nHost: api.minecraftservices.com\r\nAuthorization: Bearer token\r\n", username_to_snipe);
    let data = data.as_bytes();
    let mut config = ClientConfig::new();
    config
        .root_store
        .add_server_trust_anchors(&webpki_roots::TLS_SERVER_ROOTS);
    let connector = TlsConnector::from(Arc::new(config));
    let domain = DNSNameRef::try_from_ascii_str("api.minecraftservices.com").unwrap();
    let stream = TcpStream::connect(&addr).await.unwrap();
    let mut stream = connector.connect(domain, stream).await.unwrap();
    stream.write_all(data).await.unwrap();
    let before = Instant::now();
    stream.write_all(b"\r\n").await.unwrap();
    stream.read_exact(&mut buf).await.unwrap();
    let after = Instant::now();
    ((after - before).as_millis() as i32 - constants::SERVER_RESPONSE_TIME as i32) / 2
}

pub async fn auto_offset_calculation_gc(username_to_snipe: &str) -> i32 {
    println!("Measuring offset...");
    let mut buf = [0; 12];
    let addr = "api.minecraftservices.com:443"
        .to_socket_addrs()
        .unwrap()
        .next()
        .ok_or("failed to resolve api.minecraftservices.com")
        .unwrap();
    let post_body = json!({ "profileName": username_to_snipe }).to_string();
    let data = format!("POST /minecraft/profile HTTP/1.1\r\nHost: api.minecraftservices.com\r\nAuthorization: Bearer token\r\n\r\n{}", post_body);
    let data = data.as_bytes();
    let mut config = ClientConfig::new();
    config
        .root_store
        .add_server_trust_anchors(&webpki_roots::TLS_SERVER_ROOTS);
    let connector = TlsConnector::from(Arc::new(config));
    let domain = DNSNameRef::try_from_ascii_str("api.minecraftservices.com").unwrap();
    let stream = TcpStream::connect(&addr).await.unwrap();
    let mut stream = connector.connect(domain, stream).await.unwrap();
    stream.write_all(data).await.unwrap();
    let before = Instant::now();
    stream.write_all(b"\r\n").await.unwrap();
    stream.read_exact(&mut buf).await.unwrap();
    let after = Instant::now();
    ((after - before).as_millis() as i32 - constants::SERVER_RESPONSE_TIME as i32) / 2
}

pub async fn snipe_regular(
    snipe_time: &DateTime<Utc>,
    username_to_snipe: String,
    access_token: &str,
    spread_offset: i32,
) -> bool {
    let function_id = "SnipeRegular";
    let mut status_vec = Vec::with_capacity(constants::REGULAR_SNIPE_REQS as usize);
    let mut handle_vec = Vec::with_capacity(constants::REGULAR_SNIPE_REQS as usize);
    let mut spread = 0;
    let addr = "api.minecraftservices.com:443"
        .to_socket_addrs()
        .unwrap()
        .next()
        .ok_or("failed to resolve api.minecraftservices.com")
        .unwrap();
    let data = Arc::new(format!("PUT /minecraft/profile/name/{} HTTP/1.1\r\nHost: api.minecraftservices.com\r\nAuthorization: Bearer {}\r\n", username_to_snipe, access_token).into_bytes());
    let mut config = ClientConfig::new();
    config
        .root_store
        .add_server_trust_anchors(&webpki_roots::TLS_SERVER_ROOTS);
    let connector = Arc::new(TlsConnector::from(Arc::new(config)));
    let domain = DNSNameRef::try_from_ascii_str("api.minecraftservices.com").unwrap();
    let snipe_time = snipe_time.to_owned();
    let username_to_snipe = Arc::new(username_to_snipe);
    for _ in 0..constants::REGULAR_SNIPE_REQS {
        let connector = Arc::clone(&connector);
        let data = Arc::clone(&data);
        let username_to_snipe = Arc::clone(&username_to_snipe);
        let handle = tokio::task::spawn(async move {
            let mut buf = [0; 12];
            let snipe_time = snipe_time + Duration::milliseconds(spread);
            let handshake_time = snipe_time - Duration::seconds(5);
            let sleep_duration = match (handshake_time - Utc::now()).to_std() {
                Ok(x) => x,
                Err(_) => {
                    cli::kalm_panik(
                        function_id,
                        &format!("The name {} has already dropped.", username_to_snipe),
                    );
                    return false;
                }
            };
            sleep(sleep_duration).await;
            let stream = TcpStream::connect(&addr).await.unwrap();
            let mut stream = connector.connect(domain, stream).await.unwrap();
            stream.write_all(&data).await.unwrap();
            bunt::println!("{$green}TCP connection established!{/$}");
            let sleep_duration = match (snipe_time - Utc::now()).to_std() {
                Ok(x) => x,
                Err(_) => {
                    cli::kalm_panik(
                        function_id,
                        &format!("The name {} has already dropped.", username_to_snipe),
                    );
                    return false;
                }
            };
            sleep(sleep_duration).await;
            stream.write_all(b"\r\n").await.unwrap();
            stream.read_exact(&mut buf).await.unwrap();
            let formatted_resp_time = Utc::now().format("%F %T%.6f");
            let res = String::from_utf8_lossy(&buf);
            let status = res[9..].parse::<u16>().unwrap();
            match status {
                200 => {
                    bunt::println!(
                        "[{$green}success{/$}] {$green}200{/$} @ {[cyan]}",
                        formatted_resp_time
                    );
                    true
                }
                _ => {
                    bunt::println!(
                        "[{$red}fail{/$}] {[red]} @ {[cyan]}",
                        status,
                        formatted_resp_time
                    );
                    false
                }
            }
        });
        spread += spread_offset as i64;
        handle_vec.push(handle);
    }
    for handle in handle_vec {
        let status = handle.await.unwrap();
        status_vec.push(status);
    }
    status_vec.contains(&true)
}

pub async fn snipe_gc(
    snipe_time: &DateTime<Utc>,
    username_to_snipe: String,
    access_token: &str,
    spread_offset: i32,
) -> bool {
    let function_id = "SnipeGc";
    let mut status_vec = Vec::with_capacity(constants::GC_SNIPE_REQS as usize);
    let mut handle_vec = Vec::with_capacity(constants::GC_SNIPE_REQS as usize);
    let mut spread = 0;
    let addr = "api.minecraftservices.com:443"
        .to_socket_addrs()
        .unwrap()
        .next()
        .ok_or("failed to resolve api.minecraftservices.com")
        .unwrap();
    let post_body = json!({ "profileName": username_to_snipe }).to_string();
    let data = Arc::new(format!("POST /minecraft/profile HTTP/1.1\r\nHost: api.minecraftservices.com\r\nAccept: application/json\r\nAuthorization: Bearer {}\r\n\r\n{}", access_token, post_body).into_bytes());
    let mut config = ClientConfig::new();
    config
        .root_store
        .add_server_trust_anchors(&webpki_roots::TLS_SERVER_ROOTS);
    let connector = Arc::new(TlsConnector::from(Arc::new(config)));
    let domain = DNSNameRef::try_from_ascii_str("api.minecraftservices.com").unwrap();
    let snipe_time = snipe_time.to_owned();
    let username_to_snipe = Arc::new(username_to_snipe);
    for _ in 0..constants::GC_SNIPE_REQS {
        let connector = Arc::clone(&connector);
        let data = Arc::clone(&data);
        let username_to_snipe = Arc::clone(&username_to_snipe);
        let handle = tokio::task::spawn(async move {
            let mut buf = [0; 12];
            let snipe_time = snipe_time + Duration::milliseconds(spread);
            let handshake_time = snipe_time - Duration::seconds(5);
            let sleep_duration = match (handshake_time - Utc::now()).to_std() {
                Ok(x) => x,
                Err(_) => {
                    cli::kalm_panik(
                        function_id,
                        &format!("The name {} has already dropped.", username_to_snipe),
                    );
                    return false;
                }
            };
            sleep(sleep_duration).await;
            let stream = TcpStream::connect(&addr).await.unwrap();
            let mut stream = connector.connect(domain, stream).await.unwrap();
            stream.write_all(&data).await.unwrap();
            bunt::println!("{$green}TCP connection established!{/$}");
            let sleep_duration = match (snipe_time - Utc::now()).to_std() {
                Ok(x) => x,
                Err(_) => {
                    cli::kalm_panik(
                        function_id,
                        &format!("The name {} has already dropped.", username_to_snipe),
                    );
                    return false;
                }
            };
            sleep(sleep_duration).await;
            stream.write_all(b"\r\n").await.unwrap();
            stream.read_exact(&mut buf).await.unwrap();
            let formatted_resp_time = Utc::now().format("%F %T%.6f");
            let res = String::from_utf8_lossy(&buf);
            let status = res[9..].parse::<u16>().unwrap();
            match status {
                200 => {
                    bunt::println!(
                        "[{$green}success{/$}] {$green}200{/$} @ {[cyan]}",
                        formatted_resp_time
                    );
                    true
                }
                _ => {
                    bunt::println!(
                        "[{$red}fail{/$}] {[red]} @ {[cyan]}",
                        status,
                        formatted_resp_time
                    );
                    false
                }
            }
        });
        spread += spread_offset as i64;
        handle_vec.push(handle);
    }
    for handle in handle_vec {
        let status = handle.await.unwrap();
        status_vec.push(status);
    }
    status_vec.contains(&true)
}
