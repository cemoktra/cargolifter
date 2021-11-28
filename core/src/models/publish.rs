use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::hash::Hash;

#[derive(Clone, Debug, Default, Deserialize)]
pub struct Dependency {
    pub name: String,
    pub version_req: String,
    pub features: Vec<String>,
    pub optional: bool,
    pub default_features: bool,
    pub target: Option<String>,
    pub kind: String,
    pub registry: Option<String>,
    pub package: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct MetaData {
    pub name: String,
    pub vers: String,
    pub deps: Vec<Dependency>,
    pub features: HashMap<String, Vec<String>>,
    pub authors: Vec<String>,
    pub description: Option<String>,
    pub documentation: Option<String>,
    pub homepage: Option<String>,
    pub readme: Option<String>,
    pub readme_file: Option<String>,
    pub keywords: Vec<String>,
    pub categories: Vec<String>,
    pub license: Option<String>,
    pub license_file: Option<String>,
    pub repository: Option<String>,
    pub badges: HashMap<String, HashMap<String, String>>,
    pub links: Option<String>,
}

#[derive(Debug, Default)]
pub struct PublishRequest {
    pub meta: MetaData,
    pub data: Vec<u8>,
}

impl MetaData {
    pub fn crate_file_path(&self) -> String {
        crate::get_crate_file_path(&self.name)
    }
}

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

#[cfg(test)]
mod tests {
    #[test]
    fn test_crate_file_path_1() {
        let meta = super::MetaData {
            name: "A".into(),
            ..Default::default()
        };
        assert_eq!("1/A", meta.crate_file_path());
    }

    #[test]
    fn test_crate_file_path_2() {
        let meta = super::MetaData {
            name: "AB".into(),
            ..Default::default()
        };
        assert_eq!("2/AB", meta.crate_file_path());
    }

    #[test]
    fn test_crate_file_path_3() {
        let meta = super::MetaData {
            name: "ABC".into(),
            ..Default::default()
        };
        assert_eq!("3/A/ABC", meta.crate_file_path());
    }

    #[test]
    fn test_crate_file_path_more() {
        let meta = super::MetaData {
            name: "ABCDE".into(),
            ..Default::default()
        };
        assert_eq!("AB/CD/ABCDE", meta.crate_file_path());
    }
}
