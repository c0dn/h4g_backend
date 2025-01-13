use fred::prelude::*;
use log::{debug, error};
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};

async fn handle_error((error, _server): (Error, Option<Server>)) -> FredResult<()> {
    error!("Redis client disconnected with error: {:?}", error);
    Ok(())
}

async fn handle_reconnect(_server: Server) -> FredResult<()> {
    debug!("Redis client connected.");
    Ok(())
}

pub async fn build_redis_client() -> Client {
    let redis_auth = std::env::var("REDIS_AUTH").unwrap_or_else(|_| "".into());
    let redis_uri = std::env::var("REDIS_URI").unwrap_or_else(|_| "127.0.0.1".into());
    let config = Config::from_url(&format!("redis://{}@{}", redis_auth, redis_uri))
        .expect("Failed to parse Redis config");
    // configure exponential backoff when reconnecting, starting at 100 ms, and doubling each time up to 30 sec.
    let policy = ReconnectPolicy::new_exponential(0, 100, 30_000, 2);
    let connect_policy = ConnectionConfig::default();
    let perf = PerformanceConfig::default();
    let c = Client::new(config, Some(perf), Some(connect_policy), Some(policy));
    c.on_error(handle_error);
    c.on_reconnect(handle_reconnect);
    c.connect();

    c.wait_for_connect()
        .await
        .expect("Failed to connect to Redis");
    c
}

pub fn generate_random_string() -> String {
    const CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    let mut rng = thread_rng();
    (0..10)
        .map(|_| CHARSET[rng.gen_range(0..CHARSET.len())] as char)
        .collect()
}

pub fn generate_otp() -> String {
    let mut rng = thread_rng();
    format!("{:06}", rng.gen_range(0..999999))
}

pub fn serialize_to_messagepack<T: Serialize>(t: &T) -> Vec<u8> {
    rmp_serde::to_vec(&t).unwrap()
}

pub fn deserialize_from_messagepack<'a, T: Deserialize<'a>>(
    bytes: &'a [u8],
) -> Result<T, rmp_serde::decode::Error> {
    rmp_serde::from_slice(bytes)
}

#[macro_export]
macro_rules! regex {
    ($re:literal $(,)?) => {{
        static RE: once_cell::sync::OnceCell<regex::Regex> = once_cell::sync::OnceCell::new();
        RE.get_or_init(|| regex::Regex::new($re).unwrap())
    }};
}
