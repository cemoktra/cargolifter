use crate::web::publish::{Dependency, PublishRequest};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{collections::HashMap, hash::Hash};

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct PublishedDependency {
    pub name: String,
    pub req: String,
    pub features: Vec<String>,
    pub optional: bool,
    pub default_features: bool,
    pub target: Option<String>,
    pub kind: String,
    pub registry: Option<String>,
    pub package: Option<String>,
}

impl std::convert::From<Dependency> for PublishedDependency {
    fn from(dep: Dependency) -> Self {
        Self {
            name: dep.name,
            req: dep.version_req,
            features: dep.features,
            optional: dep.optional,
            default_features: dep.default_features,
            target: dep.target,
            kind: dep.kind,
            registry: dep.registry,
            package: dep.package,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Eq)]
pub struct PublishedVersion {
    pub name: String,
    pub vers: String,
    pub deps: Vec<PublishedDependency>,
    pub cksum: String,
    pub features: HashMap<String, Vec<String>>,
    pub yanked: bool,
    pub links: Option<String>,
}

impl std::convert::From<&PublishRequest> for PublishedVersion {
    fn from(req: &PublishRequest) -> Self {
        let cksum = Sha256::digest(&req.data);

        Self {
            name: req.meta.name.clone(),
            vers: req.meta.vers.clone(),
            deps: req
                .meta
                .deps
                .clone()
                .into_iter()
                .map(|dep| dep.into())
                .collect(),
            cksum: hex::encode(cksum),
            features: req.meta.features.clone(),
            yanked: false,
            links: req.meta.links.clone(),
        }
    }
}

impl std::cmp::PartialEq for PublishedVersion {
    fn eq(&self, other: &Self) -> bool {
        self.vers == other.vers && self.name == other.name
    }
}

impl Hash for PublishedVersion {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.vers.hash(state);
    }
}
