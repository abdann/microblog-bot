use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize)]
pub struct Post<'a> {
    pub title: &'a str,
    pub tag: &'a str,
    pub body: &'a str,
}

#[derive(Debug, Serialize)]
pub struct Login {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct PostResponse {
    pub message: String,
    pub post_url: String
}
