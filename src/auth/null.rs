use super::Authenticator;

pub struct NullAuthenticator;

impl Authenticator for NullAuthenticator {
    fn authenticate(&self, token: &str) -> bool {
        log::info!("Authenticating token '{}'", token);
        true
    }
}