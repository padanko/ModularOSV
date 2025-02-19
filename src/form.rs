use serde;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct MakeTopicFormData {
    pub title: String,
    pub name: String,
    pub body: String
}