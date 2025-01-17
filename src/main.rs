use crate::endpoint::public::serve_upload;
use crate::middleware::authentication_middleware;
use axum::http::header::{AUTHORIZATION, CONTENT_TYPE};
use axum::http::Method;
use axum::routing::get;
use axum::Router;
use axum_casbin::casbin::function_map::key_match2;
use axum_casbin::casbin::{CoreApi, DefaultModel, FileAdapter};
use axum_casbin::CasbinAxumLayer;
use diesel::Connection;
use diesel_async::async_connection_wrapper::AsyncConnectionWrapper;
use diesel_async::pooled_connection::bb8::Pool;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::AsyncPgConnection;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use dotenvy::dotenv;
use log::{info, warn};
use pasetors::keys::{AsymmetricKeyPair, Generate, SymmetricKey};
use pasetors::paserk::FormatAsPaserk;
use pasetors::version4::V4;
use socketioxide::SocketIo;
use std::sync::Arc;
use tokio::fs;
use tower::ServiceBuilder;
use tower_http::compression::CompressionLayer;
use tower_http::cors::CorsLayer;
use tower_http::decompression::RequestDecompressionLayer;
use tower_http::normalize_path::NormalizePathLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

mod backend;
mod endpoint;
mod helper;
mod middleware;
mod models;
mod paseto;
mod req_res;
mod schema;
mod utils;
mod websocket;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

#[derive(Clone)]
pub struct AppConfig {
    pub dev_mode: bool,
    pub bind_address: String,
    pub database_url: String,
}

impl AppConfig {
    fn new() -> Self {
        let dev_mode = std::env::var("DEV").unwrap_or("true".to_string()) == "true";
        let bind_address = std::env::var("HOST").unwrap_or_else(|_| {
            warn!("Listening on default address, set HOST env variable to modify");
            "localhost:5178".to_string()
        });
        let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            warn!("DATABASE_URL is not set, using default credentials");
            "postgres://postgres:password@localhost/odb_web".to_string()
        });
        AppConfig {
            dev_mode,
            bind_address,
            database_url,
        }
    }
}

#[derive(Clone)]
pub struct AppState {
    pub postgres_pool: Pool<AsyncPgConnection>,
    pub redis_client: fred::clients::Client,
    pub config: AppConfig,
}

impl AppState {
    async fn new(app_config: AppConfig) -> Self {
        let db_config =
            AsyncDieselConnectionManager::<AsyncPgConnection>::new(&app_config.database_url);
        let pool = Pool::builder()
            .build(db_config)
            .await
            .expect("Unable to create Postgres connection pool");
        let redis_client = utils::build_redis_client().await;
        AppState {
            postgres_pool: pool,
            redis_client,
            config: app_config,
        }
    }
}

async fn generate_keypair() {
    if fs::metadata("web_key.pem").await.is_ok() {
        info!("Key files already exist. Skipping generation.");
        return;
    }

    let sk_local = SymmetricKey::<V4>::generate().expect("Unable to generate local key");
    let mut sk_local_str = String::new();
    sk_local.fmt(&mut sk_local_str).unwrap();

    fs::write("web_local_key.pem", sk_local_str.as_bytes())
        .await
        .expect("Unable to save local key");

    let kp = AsymmetricKeyPair::<V4>::generate().expect("Unable to generate key pair");
    let sk = kp.secret;
    let pk = kp.public;
    let mut sk_str = String::new();
    let mut pk_str = String::new();
    sk.fmt(&mut sk_str).unwrap();
    pk.fmt(&mut pk_str).unwrap();

    fs::write("web_key.pem", sk_str.as_bytes())
        .await
        .expect("Unable to save private key");
    fs::write("web_public.pem", pk_str.as_bytes())
        .await
        .expect("Unable to save public key");
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv()?;
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let config = AppConfig::new();
    let m = DefaultModel::from_file("casbin.conf").await?;

    let a = FileAdapter::new("role_policy.csv");

    let cas_layer = CasbinAxumLayer::new(m, a).await?;

    generate_keypair().await;

    cas_layer
        .write()
        .await
        .get_role_manager()
        .write()
        .matching_fn(Some(key_match2), None);

    let app_state = AppState::new(config.clone()).await;
    let app_state = Arc::new(app_state);
    let _ = tokio::task::spawn_blocking(move || {
        let mut conn =
            AsyncConnectionWrapper::<AsyncPgConnection>::establish(&config.database_url).unwrap();
        conn.run_pending_migrations(MIGRATIONS)
            .expect("Error running migrations");
    })
    .await?;

    let origins = if config.dev_mode {
        warn!("IN DEV mode, origins CORS different");
        ["https://mwh.homelan.cc".parse()?, "http://localhost:9517".parse()?]
    } else {
        ["https://mwh.homelan.cc".parse()?, "http://localhost:9517".parse()?]
    };

    let cors_layer = CorsLayer::new()
        .allow_origin(origins)
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS, Method::PATCH])
        .allow_credentials(true)
        .allow_headers([CONTENT_TYPE, AUTHORIZATION]);


    let (ws_layer, io) = SocketIo::builder()
        .req_path("/ws")
        .with_state(app_state.clone())
        .build_layer();

    let trace_layer = TraceLayer::new_for_http();
    let normalise_path_layer = NormalizePathLayer::trim_trailing_slash();
    let service_layer = ServiceBuilder::new()
        .layer(trace_layer)
        .layer(axum::middleware::from_fn(authentication_middleware))
        .layer(RequestDecompressionLayer::new())
        .layer(CompressionLayer::new())
        .layer(cors_layer)
        .layer(cas_layer)
        .layer(normalise_path_layer);

    let app = Router::new()
        .nest("/auth", endpoint::auth::get_scope())
        .nest("/me", endpoint::me::get_scope())
        .merge(endpoint::users::get_routes())
        .merge(endpoint::products::get_routes())
        .merge(endpoint::inventory::get_routes())
        .route("/uploads/{*file}", get(serve_upload))
        .layer(ws_layer)
        .layer(service_layer)
        .with_state(app_state.clone());

    let parts = &config.bind_address.split(":").collect::<Vec<&str>>();
    let host = parts[0];
    let port = parts[1];

    info!("Server running on {host} port: {port}");

    let listener = tokio::net::TcpListener::bind(&config.bind_address).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
