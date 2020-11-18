use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

/// ``` json
/// {
///   "id": 3,
///   "description": null,
///   "default_branch": "master",
///   "visibility": "internal",
///   "ssh_url_to_repo": "git@example.com:diaspora/diaspora-project-site.git",
///   "http_url_to_repo": "http://example.com/diaspora/diaspora-project-site.git",
///   "web_url": "http://example.com/diaspora/diaspora-project-site",
///   "readme_url": "http://example.com/diaspora/diaspora-project-site/blob/master/README.md",
///   "tag_list": [
///     "example",
///     "disapora project"
///   ],
///   "name": "Diaspora Project Site",
///   "name_with_namespace": "Diaspora / Diaspora Project Site",
///   "path": "diaspora-project-site",
///   "path_with_namespace": "diaspora/diaspora-project-site",
///   "issues_enabled": true,
///   "open_issues_count": 1,
///   "merge_requests_enabled": true,
///   "jobs_enabled": true,
///   "wiki_enabled": true,
///   "snippets_enabled": false,
///   "can_create_merge_request_in": true,
///   "resolve_outdated_diff_discussions": false,
///   "container_registry_enabled": false,
///   "created_at": "2013-09-30T13:46:02Z",
///   "last_activity_at": "2013-09-30T13:46:02Z",
///   "creator_id": 3,
///   "namespace": {
///     "id": 3,
///     "name": "Diaspora",
///     "path": "diaspora",
///     "kind": "group",
///     "full_path": "diaspora"
///   },
///   "import_status": "none",
///   "archived": true,
///   "avatar_url": "http://example.com/uploads/project/avatar/3/uploads/avatar.png",
///   "shared_runners_enabled": true,
///   "forks_count": 0,
///   "star_count": 1,
///   "public_jobs": true,
///   "shared_with_groups": [],
///   "only_allow_merge_if_pipeline_succeeds": false,
///   "allow_merge_on_skipped_pipeline": false,
///   "only_allow_merge_if_all_discussions_are_resolved": false,
///   "remove_source_branch_after_merge": false,
///   "request_access_enabled": false,
///   "merge_method": "merge",
///   "autoclose_referenced_issues": true,
///   "suggestion_commit_message": null,
///   "_links": {
///     "self": "http://example.com/api/v4/projects",
///     "issues": "http://example.com/api/v4/projects/1/issues",
///     "merge_requests": "http://example.com/api/v4/projects/1/merge_requests",
///     "repo_branches": "http://example.com/api/v4/projects/1/repository_branches",
///     "labels": "http://example.com/api/v4/projects/1/labels",
///     "events": "http://example.com/api/v4/projects/1/events",
///     "members": "http://example.com/api/v4/projects/1/members"
///   }
/// }
/// ```
#[derive(Debug, Serialize, Deserialize)]
pub struct Fork {
    pub id: usize,
    pub description: Option<String>,
    pub default_branch: String,
    pub visibility: String,
    pub ssh_url_to_repo: String,
    pub http_url_to_repo: String,
    pub web_url: String,
    pub readme_url: String,
    pub tag_list: Vec<String>,
    pub name: String,
    pub name_with_namespace: String,
    pub path: String,
    pub path_with_namespace: String,
    pub issues_enabled: bool,
    pub open_issues_count: usize,
    pub merge_requests_enabled: bool,
    pub jobs_enabled: bool,
    pub wiki_enabled: bool,
    pub snippets_enabled: bool,
    pub can_create_merge_request_in: bool,
    pub resolve_outdated_diff_discussions: bool,
    pub container_registry_enabled: bool,
    pub created_at: String,
    pub last_activity_at: String,
    pub creator_id: usize,
    pub namespace: Namespace,
    pub import_status: String,
    pub archived: bool,
    pub avatar_url: Option<String>,
    pub shared_runners_enabled: bool,
    pub forks_count: usize,
    pub star_count: usize,
    pub public_jobs: bool,
    pub shared_with_groups: Vec<String>,
    pub only_allow_merge_if_pipeline_succeeds: bool,
    pub allow_merge_on_skipped_pipeline: Option<bool>,
    pub only_allow_merge_if_all_discussions_are_resolved: bool,
    pub remove_source_branch_after_merge: bool,
    pub request_access_enabled: bool,
    pub merge_method: String,
    pub autoclose_referenced_issues: bool,
    pub suggestion_commit_message: Option<String>,
    pub _links: Links,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Links {
    #[serde(rename = "self")]
    _self: String,
    issues: String,
    merge_requests: String,
    repo_branches: String,
    labels: String,
    events: String,
    members: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Namespace {
    pub id: usize,
    pub name: String,
    pub path: String,
    pub kind: String,
    pub full_path: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Forks(Vec<Fork>);

impl IntoIterator for Forks {
    type Item = Fork;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl Forks {
    pub async fn get<G: AsRef<str> + Display>(
        client: &Client,
        gitlab_api_url: G,
        project_id: usize,
    ) -> anyhow::Result<Forks> {
        let mut list = Vec::new();

        for page_id in 1..10 as usize {
            let res = client
                .get(&format!(
                    "{}/v4/projects/{}/forks",
                    &gitlab_api_url, &project_id
                ))
                .query(&[
                    ("order_by", "created_at"),
                    ("per_page", "100"),
                    ("sort", "asc"),
                    ("page", page_id.to_string().as_str()),
                ])
                .send()
                .await?;

            let data = res.text().await?.replace("\n", " ");
            let current: Vec<Fork> = serde_json::from_str(&data)?;

            if current.is_empty() {
                break;
            }

            list.extend(current);
        }

        Ok(Forks(list))
    }
}
