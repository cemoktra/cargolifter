use git2::{Cred, RemoteCallbacks};

pub fn init_auth_callback() -> RemoteCallbacks<'static> {
    let mut callbacks = RemoteCallbacks::new();
    callbacks.credentials(move |_url, username_from_url, _allowed_types| {
        Cred::ssh_key_from_agent(username_from_url.unwrap())
    });
    callbacks
}
