// use fermi::Atom;
use serde::{Deserialize, Serialize};

pub mod app;
pub mod auth_form;
pub mod health_check;

pub const API_URL: &str = "http://localhost:3000";

// pub static USER_ID: Atom<Option<&str>> = |_| None;

// TODO: Import these from `alloxid_http`.
#[derive(Debug, Deserialize, Serialize)]
pub struct UserCreatedData {
    pub id: String,
    pub token: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct JsonBody<T> {
    pub data: T,
}
