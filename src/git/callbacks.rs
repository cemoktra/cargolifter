use std::path::Path;

use git2::{Cred, RemoteCallbacks};

pub fn init_auth_callback(public_key_path: &str) -> RemoteCallbacks {
    let mut callbacks = RemoteCallbacks::new();
    let path = String::from(public_key_path);
    callbacks.credentials(move |_url, username_from_url, _allowed_types| {
        Cred::ssh_key(username_from_url.unwrap(), None, Path::new(&path), None)
    });
    callbacks
}
