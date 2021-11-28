mod endpoints;

use hyper::body::Buf;

pub struct WebService {
    backend: tokio::sync::mpsc::Sender<cargolifter_core::BackendCommand>,
    storage: tokio::sync::mpsc::Sender<cargolifter_core::StorageCommand>,
    config: cargolifter_core::config::WebServiceConfig,
}

pub struct RequestExtractor(cargolifter_core::models::PublishRequest);

#[async_trait::async_trait]
impl axum::extract::FromRequest for RequestExtractor {
    type Rejection = axum::http::StatusCode;

    async fn from_request(req: &mut axum::extract::RequestParts) -> Result<Self, Self::Rejection> {
        let mut data = hyper::body::to_bytes(req.body_mut().unwrap())
            .await
            .unwrap();
        let json_length = data.get_u32_le() as usize;
        let json_data = &data[0..json_length].to_vec();
        data.advance(json_length);
        let data_length = data.get_u32_le() as usize;

        Ok(Self {
            0: cargolifter_core::models::PublishRequest {
                meta: serde_json::from_slice(json_data).unwrap(),
                data: data[0..data_length].to_vec(),
            },
        })
    }
}

impl WebService {
    pub fn new(
        backend: tokio::sync::mpsc::Sender<cargolifter_core::BackendCommand>,
        storage: tokio::sync::mpsc::Sender<cargolifter_core::StorageCommand>,
        config: cargolifter_core::config::WebServiceConfig,
    ) -> Self {
        Self {
            backend,
            storage,
            config,
        }
    }

    pub async fn run(&self) {
        let host = format!("0.0.0.0:{}", self.config.port); // TODO: confiure port
        tracing::info!("starting web service at: {}", host);

        let app = axum::Router::new()
            .route(
                "/api/v1/crates/:crate_name/:crate_version/download",
                axum::handler::get(endpoints::download),
            )
            .route("/api/v1/crates/new", axum::handler::put(endpoints::publish))
            .route(
                "/api/v1/crates/:name/:version/yank",
                axum::handler::delete(endpoints::yank),
            )
            .route(
                "/api/v1/crates/:name/:version/unyank",
                axum::handler::put(endpoints::unyank),
            )
            .layer(axum::AddExtensionLayer::new(self.backend.clone()))
            .layer(axum::AddExtensionLayer::new(self.storage.clone()));

        axum::Server::bind(&host.parse().unwrap())
            .serve(app.into_make_service())
            .await
            .unwrap();
    }
}
