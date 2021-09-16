use crate::config::git::GitConfig;
use git2::{Error, MergeOptions, Repository, Signature};
use std::sync::{Arc, Mutex};

pub struct GitService {
    repo: Repository,
    config: GitConfig,
}

impl GitService {
    pub fn from_config(config: &GitConfig) -> Result<Arc<Mutex<GitService>>, Error> {
        let mut service = GitService {
            repo: GitService::init(config)?,
            config: config.clone(),
        };
        service.configure(config)?;
        Ok(Arc::new(Mutex::new(service)))
    }

    fn configure(&mut self, registry_config: &GitConfig) -> Result<(), Error> {
        let mut config = self.repo.config()?.open_level(git2::ConfigLevel::Local)?;
        config.set_str(
            "user.email",
            &registry_config
                .email
                .as_ref()
                .unwrap_or(&String::from("crate@cargolifter.com")),
        )?;
        config.set_str(
            "user.name",
            &registry_config
                .username
                .as_ref()
                .unwrap_or(&String::from("cargolifter")),
        )?;
        Ok(())
    }

    fn init(config: &GitConfig) -> Result<Repository, Error> {
        match Repository::open(&config.clone_path) {
            Ok(repo) => {
                // TODO: check if cargo repo
                return Ok(repo);
            }
            _ => {}
        };

        Repository::clone(&config.remote_url, &config.clone_path)
    }

    pub fn pull_crates_io(&self) -> Result<(), Error> {
        let signature = Signature::now(
            self.config
                .username
                .as_ref()
                .unwrap_or(&String::from("cargolifter")),
            self.config
                .email
                .as_ref()
                .unwrap_or(&String::from("git@cargolifter.com")),
        )?;

        let mut remote = match self.repo.find_remote("crates.io") {
            Ok(remote) => remote,
            Err(_) => self.repo.remote(
                "crates.io",
                "https://github.com/rust-lang/crates.io-index.git",
            )?,
        };
        log::info!(
            "fetching master from crates.io [{}]",
            remote.url().unwrap_or_default()
        );
        remote.fetch(&["master"], None, None)?;
        let fetch_head = self.repo.find_reference("FETCH_HEAD")?;
        let fetch_commit = self.repo.reference_to_annotated_commit(&fetch_head)?;

        let local_head = self.repo.head()?;
        let local_commit = self.repo.reference_to_annotated_commit(&local_head)?;
        let local_commit = self.repo.find_commit(local_commit.id())?;

        let mut merge_options = MergeOptions::new();
        merge_options.file_favor(git2::FileFavor::Ours);
        log::info!("merging '{}' into refs/heads/master", fetch_commit.id());
        self.repo
            .merge(&[&fetch_commit], Some(&mut merge_options), None)?;
        log::info!(
            "successfully merged {} into refs/heads/master",
            fetch_commit.id()
        );
        self.repo.cleanup_state()?;

        let tree = self.repo.find_tree(self.repo.index()?.write_tree()?)?;

        let commit = self.repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            &format!("crates.io sync of {}", chrono::offset::Utc::now()),
            &tree,
            &[&local_commit],
        )?;
        log::info!("committed sync with '{}'", commit);

        let mut origin = self.repo.find_remote("origin")?;
        log::info!("pushing to origin [{}]", origin.url().unwrap_or_default());
        origin.push(&["refs/heads/master:refs/heads/master"], None)?;
        log::info!("successfully pushed to origin");

        Ok(())
    }
}
