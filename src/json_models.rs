use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Post<'a> {
    pub title: &'a str,
    pub tag: &'a str,
    pub body: &'a str,
}

#[derive(Debug, Serialize)]
pub struct Login<'a> {
    pub username: &'a str,
    pub password: &'a str,
}
