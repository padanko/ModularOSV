use serde;
use actix_multipart::form::{json::Json as MPJson, tempfile::TempFile, MultipartForm};

#[derive(serde::Deserialize)]
pub struct MakeTopicFormData {
    pub title: String,
    pub name: String,
    pub body: String
}

#[derive(serde::Deserialize)]
pub struct MakePostFormData {
    pub topicid: String,
    pub name: String,
    pub body: String
}

#[derive(Debug, MultipartForm)]
pub struct UploadForm {
    #[multipart(limit = "100MB")]
    pub file: TempFile,
}