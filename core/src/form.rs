use actix_multipart::form::{tempfile::TempFile, MultipartForm};

#[derive(serde::Deserialize)]
pub struct MakeTopicFormData {
    pub title: String,
    pub name: String,
    pub body: String,
}

#[derive(serde::Deserialize)]
pub struct MakePostFormData {
    pub topicid: String,
    pub name: String,
    pub body: String,
}

#[derive(Debug, MultipartForm)]
pub struct UploadForm {
    #[multipart(limit = "20MB")]
    pub file: TempFile,
}

#[derive(serde::Deserialize)]
pub struct FileSearchFormData {
    pub query: String
}

#[derive(serde::Deserialize)]
pub struct PostSearchQuery {
    pub query: String
}