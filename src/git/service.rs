use crate::git::callbacks::init_auth_callback;
use crate::{config::git::GitConfig, tools::crate_name_to_path, web::publish::PublishRequest};
use git2::build::RepoBuilder;
use git2::{
    AnnotatedCommit, Error, FetchOptions, IndexAddOption, MergeOptions, PushOptions, Repository,
    Signature,
};
use std::io::Write;
use std::path::Path;
use std::{
    collections::HashSet,
    io::{BufRead, BufReader, BufWriter},
    path::PathBuf,
};
use tokio::sync::mpsc::Sender;
use tokio::task::JoinHandle;

use super::model::PublishedVersion;

pub enum GitMirrorCommand {
    Mirror(tokio::sync::oneshot::Sender<bool>),
}

pub struct GitMirror;

impl GitMirror {
    pub fn run(config: &GitConfig) -> (JoinHandle<()>, Sender<GitMirrorCommand>) {
        let (sender, mut receiver) = tokio::sync::mpsc::channel::<GitMirrorCommand>(16);

        // TODO: this unwrap is bad
        let repo = GitRepo::from_config(config).unwrap();
        let handle = tokio::spawn(async move {
            loop {
                match receiver.recv().await {
                    Some(command) => match command {
                        GitMirrorCommand::Mirror(result_sender) => match repo.pull_crates_io() {
                            Ok(_) => match result_sender.send(true) {
                                Ok(_) => {}
                                Err(_) => {
                                    log::error!("Failed to send mirror result!");
                                }
                            },
                            Err(e) => {
                                log::error!("Publish failed: {}", e);
                                match result_sender.send(false) {
                                    Ok(_) => {}
                                    Err(_) => {
                                        log::error!("Failed to send mirror result!");
                                    }
                                }
                            }
                        },
                    },
                    None => {
                        log::warn!("Did not receive a GitMirrorCommand!")
                    }
                }
            }
        });
        (handle, sender)
    }
}

pub enum GitRegistryCommand {
    Publish(PublishRequest, tokio::sync::oneshot::Sender<bool>),
    Yank(YankRequest),
}

pub struct YankRequest {
    pub crate_name: String,
    pub crate_version: String,
    pub yank: bool,
    pub result_sender: tokio::sync::oneshot::Sender<bool>,
}

pub struct GitRegistry;

impl GitRegistry {
    pub fn run(config: &GitConfig) -> (JoinHandle<()>, Sender<GitRegistryCommand>) {
        let (sender, mut receiver) = tokio::sync::mpsc::channel::<GitRegistryCommand>(16);

        let repo = GitRepo::from_config(config).unwrap();
        let handle = tokio::spawn(async move {
            loop {
                match receiver.recv().await {
                    Some(command) => match command {
                        GitRegistryCommand::Publish(request, result_sender) => {
                            match repo.publish(&request) {
                                Ok(_) => match result_sender.send(true) {
                                    Ok(_) => {}
                                    Err(_) => {
                                        log::error!("Failed to send publish result!");
                                    }
                                },
                                Err(e) => {
                                    log::error!("Publish failed: {}", e);
                                    match result_sender.send(false) {
                                        Ok(_) => {}
                                        Err(_) => {
                                            log::error!("Failed to send publish result!");
                                        }
                                    }
                                }
                            }
                        }
                        GitRegistryCommand::Yank(request) => {
                            match repo.yank(
                                request.yank,
                                &request.crate_name,
                                &request.crate_version,
                            ) {
                                Ok(_) => match request.result_sender.send(true) {
                                    Ok(_) => {}
                                    Err(_) => {
                                        log::error!("Failed to send yank result!");
                                    }
                                },
                                Err(e) => {
                                    log::error!("Publish failed: {}", e);
                                    match request.result_sender.send(false) {
                                        Ok(_) => {}
                                        Err(_) => {
                                            log::error!("Failed to send yank result!");
                                        }
                                    }
                                }
                            }
                        }
                    },
                    None => {
                        log::warn!("Did not receive a GitRegistryCommand!")
                    }
                }
            }
        });
        (handle, sender)
    }
}

pub struct GitRepo {
    repo: Repository,
    config: GitConfig,
}

impl GitRepo {
    pub fn from_config(config: &GitConfig) -> Result<GitRepo, Error> {
        let mut service = GitRepo {
            repo: GitRepo::init(config)?,
            config: config.clone(),
        };
        service.configure(config)?;
        Ok(service)
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

        let mut fetch_options = FetchOptions::new();
        fetch_options.remote_callbacks(init_auth_callback());
        let mut builder = RepoBuilder::new();
        builder.fetch_options(fetch_options);
        builder.clone(&config.remote_url, Path::new(&config.clone_path))
    }

    fn signature(&self) -> Result<Signature, Error> {
        Signature::now(
            self.config
                .username
                .as_ref()
                .unwrap_or(&String::from("cargolifter")),
            self.config
                .email
                .as_ref()
                .unwrap_or(&String::from("git@cargolifter.com")),
        )
    }

    fn commit(&self, message: &str) -> Result<(), Error> {
        let signature = self.signature()?;

        let local_head = self.repo.head()?;
        let local_commit = self.repo.reference_to_annotated_commit(&local_head)?;
        let local_commit = self.repo.find_commit(local_commit.id())?;

        let tree = self.repo.find_tree(self.repo.index()?.write_tree()?)?;

        let commit = self.repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            message,
            &tree,
            &[&local_commit],
        )?;

        log::info!("committed '{}' with '{}'", message, commit);

        Ok(())
    }

    fn push(&self, remote_name: &str) -> Result<(), Error> {
        let branch = self
            .config
            .branch
            .as_ref()
            .unwrap_or(&String::from("master"))
            .clone();
        let mut remote = self.repo.find_remote(remote_name)?;
        log::info!(
            "pushing to {} [{}]",
            remote_name,
            remote.url().unwrap_or_default()
        );

        let mut push_options = PushOptions::new();
        push_options.remote_callbacks(init_auth_callback());
        remote.push(
            &[format!("refs/heads/{}:refs/heads/{}", branch, branch)],
            Some(&mut push_options),
        )?;
        log::info!("successfully pushed to {}", remote_name);
        Ok(())
    }

    fn merge(&self, merge_commit: &AnnotatedCommit) -> Result<(), Error> {
        let branch = self
            .config
            .branch
            .as_ref()
            .unwrap_or(&String::from("master"))
            .clone();
        let mut merge_options = MergeOptions::new();
        merge_options.file_favor(git2::FileFavor::Ours);
        log::info!("merging '{}' into refs/heads/{}", merge_commit.id(), branch);
        self.repo
            .merge(&[&merge_commit], Some(&mut merge_options), None)?;
        log::info!(
            "successfully merged {} into refs/heads/{}",
            merge_commit.id(),
            branch
        );
        self.repo.cleanup_state()?;

        Ok(())
    }

    fn read_versions(&self, crate_path: &PathBuf) -> Result<HashSet<PublishedVersion>, Error> {
        match std::fs::File::open(&crate_path) {
            Ok(file) => {
                let mut file = BufReader::new(file);
                let mut published_versions = HashSet::new();
                loop {
                    let mut line = String::new();
                    let read = file.read_line(&mut line).unwrap();
                    if read == 0 {
                        break;
                    }
                    let published_version: PublishedVersion = serde_json::from_str(&line).unwrap();
                    published_versions.insert(published_version);
                }

                Ok(published_versions)
            }
            Err(_) => Ok(HashSet::new()),
        }
    }

    fn write_versions(
        &self,
        crate_path: &PathBuf,
        published_versions: &HashSet<PublishedVersion>,
    ) -> Result<(), Error> {
        let file = std::fs::File::create(crate_path).unwrap();
        let mut file = BufWriter::new(file);
        for version in published_versions {
            let json = serde_json::to_string(&version).unwrap();
            file.write(json.as_bytes()).unwrap();
            file.write(&['\n' as u8]).unwrap();
        }

        Ok(())
    }

    fn crate_path(&self, crate_name: &str) -> PathBuf {
        let crate_path = crate_name_to_path(crate_name);
        let mut crate_path = self.repo.workdir().unwrap().join(crate_path);
        std::fs::create_dir_all(crate_path.clone()).unwrap();
        crate_path.push(crate_name);
        crate_path
    }

    pub fn pull_crates_io(&self) -> Result<(), Error> {
        let branch = self
            .config
            .branch
            .as_ref()
            .unwrap_or(&String::from("master"))
            .clone();
        let mut remote = match self.repo.find_remote("crates.io") {
            Ok(remote) => remote,
            Err(_) => self.repo.remote(
                "crates.io",
                "https://github.com/rust-lang/crates.io-index.git",
            )?,
        };
        log::info!(
            "fetching {} from crates.io [{}]",
            branch,
            remote.url().unwrap_or_default()
        );
        let mut fetch_options = FetchOptions::new();
        fetch_options.remote_callbacks(init_auth_callback());
        remote.fetch(&[branch], Some(&mut fetch_options), None)?;

        let fetch_head = self.repo.find_reference("FETCH_HEAD")?;
        let fetch_commit = self.repo.reference_to_annotated_commit(&fetch_head)?;
        self.merge(&fetch_commit)?;
        self.commit(&format!("crates.io sync of {}", chrono::offset::Utc::now()))?;
        self.push("origin")
    }

    pub fn publish(&self, request: &PublishRequest) -> Result<(), Error> {
        let crate_path = self.crate_path(&request.meta.name);

        log::info!("reading existing versions of crate '{}'", request.meta.name);
        let mut published_versions = self.read_versions(&crate_path)?;
        log::info!(
            "found {} versions of crate '{}'",
            published_versions.len(),
            request.meta.name
        );
        published_versions.insert(request.clone().into());

        self.write_versions(&crate_path, &published_versions)?;

        let mut index = self.repo.index().unwrap();
        index
            .add_all(["*"].iter(), IndexAddOption::DEFAULT, None)
            .unwrap();
        index.write().unwrap();

        self.commit(&format!(
            "published {} version {}",
            request.meta.name, request.meta.vers
        ))?;
        self.push("origin")
    }

    pub fn yank(&self, yank: bool, crate_name: &str, crate_version: &str) -> Result<(), Error> {
        let crate_path = self.crate_path(&crate_name);

        log::info!("reading existing versions of crate '{}'", crate_name);
        let published_versions = self.read_versions(&crate_path)?;
        log::info!(
            "found {} versions of crate '{}'",
            published_versions.len(),
            crate_name
        );

        let mut updated_versions = HashSet::new();
        for published_version in published_versions {
            let mut updated_version = published_version;
            if updated_version.vers == crate_version {
                updated_version.yanked = yank;
            }
            updated_versions.insert(updated_version);
        }

        self.write_versions(&crate_path, &updated_versions)?;

        let mut index = self.repo.index().unwrap();
        index
            .add_all(["*"].iter(), IndexAddOption::DEFAULT, None)
            .unwrap();
        index.write().unwrap();

        self.commit(&format!(
            "published {} version {}",
            crate_name, crate_version
        ))?;
        self.push("origin")
    }
}
