mod endpoints;

use hyper::body::Buf;

pub struct WebService {
    backend: tokio::sync::mpsc::Sender<cargolifter_core::BackendCommand>,
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
    pub fn new(backend: tokio::sync::mpsc::Sender<cargolifter_core::BackendCommand>) -> Self {
        Self { backend }
    }

    pub async fn run(&self) {
        let host = format!("0.0.0.0:{}", 8080); // TODO: confiure port
        tracing::info!("starting web service at: {}", host);

        let app = axum::Router::new()
            .route("/api/v1/crates/new", axum::handler::put(endpoints::publish))
            .layer(axum::AddExtensionLayer::new(self.backend.clone()));

        axum::Server::bind(&host.parse().unwrap())
            .serve(app.into_make_service())
            .await
            .unwrap();
    }
}
