use reqwest::Client;
use serde::{Deserialize, Serialize, Serializer};
use std::fmt::Display;

#[derive(Debug, Serialize)]
pub struct NewIssueRequest {
    pub title: String,
    pub description: String,
    #[serde(serialize_with = "labels_serialize")]
    pub labels: Vec<String>,
}

fn labels_serialize<S>(labels: &Vec<String>, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_str(&labels.join(","))
}

impl NewIssueRequest {
    pub async fn post<G: AsRef<str> + Display>(
        self,
        client: &Client,
        gitlab_api_url: G,
        project_id: u32,
    ) -> anyhow::Result<NewIssueResponse> {
        Ok(client
            .post(&format!(
                "{}/projects/{}/issues",
                gitlab_api_url, project_id
            ))
            .json(&self)
            .send()
            .await?
            .json()
            .await?)
    }
}

/// {
///   "project_id" : 4,
///   "id" : 84,
///   "created_at" : "2016-01-07T12:44:33.959Z",
///   "iid" : 14,
///   "title" : "Issues with auth",
///   "state" : "opened"
///   ...
/// }
#[derive(Debug, Deserialize)]
pub struct NewIssueResponse {
    state: String,
}

impl NewIssueResponse {
    pub fn is_opened(&self) -> bool {
        self.state == "opened"
    }
}
