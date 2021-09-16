use tower_web::*;
use serde::Serialize;
use std::sync::{Arc, Mutex};


pub struct RegistryService {
    repo: Arc<Mutex<git2::Repository>>
}

impl RegistryService {
    pub fn new(repo: Arc<Mutex<git2::Repository>>) -> RegistryService {
        RegistryService {
            repo
        }
    }
}

#[derive(Response)]
struct MeResponse {
    message: &'static str,
}

#[derive(Response)]
struct QueryResponse {
    crates: Vec<CrateResponse>,
    meta: MetaResponse
}

#[derive(Response)]
struct NewResponse {
    warnings: Vec<Warning>
}

#[derive(Response)]
struct SuccessResponse {
    ok: bool,
    msg: Option<String>
}

#[derive(Response)]
struct UsersResponse {
    users: Vec<User>
}

#[derive(Serialize, Debug)]
struct CrateResponse {
    name: &'static str,
    max_version: &'static str,
    description: &'static str
}

#[derive(Serialize, Debug)]
struct MetaResponse {
    total: i32
}

#[derive(Serialize, Debug)]
struct Warning {
    invalid_categories: Vec<String>,
    invalid_badges: Vec<String>,
    other: Vec<String>,
}

#[derive(Serialize, Debug)]
struct User {
    id: i32,
    login: String,
    name: String,
}


fn parse_query_string(query_string: &str) -> (&str, i32) {
    let mut query = "";
    let mut per_page = 10;

    for item in query_string.split("&").collect::<Vec<&str>>() {
        if item.len() == 0 {
            break;
        }
        let split = item.split("=").collect::<Vec<&str>>();
        let mut it = split.iter();
        let key = it.next().unwrap();
        let value = it.next().unwrap();

        match *key {
            "q" => query = value,
            "per_page" => per_page = value.parse().expect("per_page must be a integral number"),
            _ => {}
        };
    }
    (query, per_page)
}

impl_web! {
    impl RegistryService {
        #[get("/me")]
        #[content_type("json")]
        fn me(&self) -> Result<MeResponse, ()> {
            Ok(MeResponse {
                message: "hello world",
            })
        }

        #[get("/api/v1/crates")]
        #[content_type("json")]
        fn query(&self, query_string: String) -> Result<QueryResponse, ()> {
            let (query, per_page) = parse_query_string(&query_string);
            println!("query: {}, per page: {}", query, per_page);
            // TODO: implement search
            Ok(QueryResponse {
                crates: Vec::new(),
                meta: MetaResponse {
                    total: 0
                }
            })
        }

        #[get("/api/v1/crates/:cratename/:version/download")]
        #[content_type("json")]
        fn download(&self, _cratename: String, _version: String) -> Result<QueryResponse, ()> {
            todo!()
        }

        #[put("/api/v1/crates/new")]
        #[content_type("json")]
        fn publish(&self) -> Result<NewResponse, ()> {
            todo!()
        }

        #[delete("/api/v1/crates/:cratename/:version/yank")]
        #[content_type("json")]
        fn yank(&self, _cratename: String, _version: String) -> Result<SuccessResponse, ()> {
            todo!()
        }

        #[put("/api/v1/crates/:cratename/:version/unyank")]
        #[content_type("json")]
        fn unyank(&self, _cratename: String, _version: String) -> Result<SuccessResponse, ()> {
            todo!()
        }

        #[get("/api/v1/crates/:cratename/owners")]
        #[content_type("json")]
        fn get_owners(&self, _cratename: String) -> Result<UsersResponse, ()> {
            todo!()
        }

        #[put("/api/v1/crates/:cratename/owners")]
        #[content_type("json")]
        fn add_owner(&self, _cratename: String) -> Result<SuccessResponse, ()> {
            todo!()
        }

        #[delete("/api/v1/crates/:cratename/owners")]
        #[content_type("json")]
        fn remove_owner(&self, _cratename: String) -> Result<SuccessResponse, ()> {
            todo!()
        }
    }
}