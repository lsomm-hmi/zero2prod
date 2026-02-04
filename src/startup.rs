use crate::configuration::{DatabaseSettings, Settings};
use crate::email_client::EmailClient;
use crate::routes::{confirm, health_check, subscribe};
use crate::state::AppState;
use axum::{
    Router,
    routing::{get, post},
};
use http::Request;
use sqlx::{Pool, Postgres, postgres::PgPoolOptions};
use std::time::Duration;
use tokio::net::TcpListener;
use tower_http::{
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, RequestId, SetRequestIdLayer},
    trace::TraceLayer,
};

pub struct Application {
    listener: TcpListener,
    state: AppState,
    port: u16,
}

impl Application {
    pub async fn build(config: Settings) -> Result<Self, std::io::Error> {
        let db = get_connection_pool(&config.database);

        let sender_email = config
            .email_client
            .sender()
            .expect("Invalid sender email address.");
        let timeout = config.email_client.timeout();
        let email_client = EmailClient::new(
            config.email_client.base_url,
            sender_email,
            config.email_client.auth_token,
            timeout,
        );

        let state = AppState { db, email_client };

        let addr = format!("{}:{}", config.application.host, config.application.port);
        let listener = TcpListener::bind(addr).await?;
        let port = listener.local_addr().unwrap().port();

        Ok(Self {
            listener,
            state,
            port,
        })
    }

    // This is useful because when the port in config is 0, a random port will be assigned
    // which we need to know post hoc.
    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn define_router(state: AppState) -> Router {
        Router::new()
            .route("/health_check", get(health_check))
            .route("/subscriptions", post(subscribe))
            .route("/subscriptions/confirm", get(confirm))
            .with_state(state)
            .layer(
                TraceLayer::new_for_http().make_span_with(|request: &Request<_>| {
                    let request_id = request
                        .extensions()
                        .get::<RequestId>()
                        .and_then(|id| id.header_value().to_str().ok())
                        .unwrap_or("unknown");

                    tracing::info_span!(
                        "http_request",
                        method = %request.method(),
                        uri = %request.uri(),
                        request_id = %request_id,
                    )
                }),
            )
            .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
            .layer(PropagateRequestIdLayer::x_request_id())
    }

    pub async fn run(self) -> Result<(), std::io::Error> {
        let Application {
            listener,
            state,
            port: _,
        } = self;
        let router = Self::define_router(state);
        axum::serve(listener, router.into_make_service()).await
    }
}

pub fn get_connection_pool(db_config: &DatabaseSettings) -> Pool<Postgres> {
    PgPoolOptions::new()
        .acquire_timeout(Duration::from_secs(10))
        .connect_lazy_with(db_config.with_db())
}
