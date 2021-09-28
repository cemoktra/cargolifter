use crate::{
    config::git::GitConfig, git::model::PublishedVersion, tools::crate_name_to_path,
    web::publish::PublishRequest,
};
use git2::{AnnotatedCommit, Error, IndexAddOption, MergeOptions, Repository, Signature};
use std::io::Write;
use std::{
    collections::HashSet,
    io::{BufRead, BufReader, BufWriter},
    path::PathBuf,
    sync::{Arc, Mutex},
};

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
        let mut remote = self.repo.find_remote(remote_name)?;
        log::info!(
            "pushing to {} [{}]",
            remote_name,
            remote.url().unwrap_or_default()
        );
        remote.push(&["refs/heads/master:refs/heads/master"], None)?;
        log::info!("successfully pushed to {}", remote_name);
        Ok(())
    }

    fn merge(&self, merge_commit: &AnnotatedCommit) -> Result<(), Error> {
        let mut merge_options = MergeOptions::new();
        merge_options.file_favor(git2::FileFavor::Ours);
        log::info!("merging '{}' into refs/heads/master", merge_commit.id());
        self.repo
            .merge(&[&merge_commit], Some(&mut merge_options), None)?;
        log::info!(
            "successfully merged {} into refs/heads/master",
            merge_commit.id()
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
        published_versions.insert(request.meta.clone().into());

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
        let crate_path = self.crate_path(&crate_version);

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
