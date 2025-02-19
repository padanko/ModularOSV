#[derive(serde::Serialize, serde::Deserialize)]
pub struct Post {
    name: String,
    body: String,
    date: String,
    ip: String,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Topic {
    title: String,
    thread_admin: String,
    contents: Vec<Post>,

}