use cargolifter_backend_gitlab::Gitlab;
use cargolifter_core::BackendService;
use cargolifter_web::WebService;


#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().init();
    
    let gitlab = Gitlab::new(31551730);
    let backend = BackendService::new(gitlab);

    let (backend_handle, backend_sender) = backend.run();

    let web = WebService::new(backend_sender);
    web.run().await;
    let _ = futures::join!(backend_handle);
}
