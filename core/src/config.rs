/// Backend configuration
pub mod backend;
/// Cargolifter configuration
pub mod cargolifter;
/// FileSystem storage configuration
pub mod filesystem;
/// Github backend configuration
pub mod github;
/// Gitlab backend configuration
pub mod gitlab;
/// S3 storage configuration
pub mod s3;
/// Storage configuration
pub mod storage;
/// Web service configuration
pub mod web;

pub use backend::*;
pub use cargolifter::*;
pub use filesystem::*;
pub use github::*;
pub use gitlab::*;
pub use s3::*;
pub use storage::*;
pub use web::*;
