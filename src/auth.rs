pub mod null;


pub trait Authenticator {
    fn authenticate(&self, token: &str) -> bool;
}