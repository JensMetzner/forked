use reqwest::Client;
use serde::{Deserialize, Serialize};

/// ``` json
/// [
///   {
///     "id": 1,
///     "username": "raymond_smith",
///     "name": "Raymond Smith",
///     "state": "active",
///     "avatar_url": "https://www.gravatar.com/avatar/c2525a7f58ae3776070e44c106c48e15?s=80&d=identicon",
///     "web_url": "http://192.168.1.8:3000/root",
///     "expires_at": "2012-10-22T14:13:35Z",
///     "access_level": 30,
///     "group_saml_identity": null
///   },
///   {
///     "id": 2,
///     "username": "john_doe",
///     "name": "John Doe",
///     "state": "active",
///     "avatar_url": "https://www.gravatar.com/avatar/c2525a7f58ae3776070e44c106c48e15?s=80&d=identicon",
///     "web_url": "http://192.168.1.8:3000/root",
///     "expires_at": "2012-10-22T14:13:35Z",
///     "access_level": 30,
///     "email": "john@example.com",
///     "group_saml_identity": {
///       "extern_uid":"ABC-1234567890",
///       "provider": "group_saml",
///       "saml_provider_id": 10
///     }
///   }
/// ]
/// ```
#[derive(Debug, Serialize, Deserialize)]
pub struct Member {
    pub username: String,
    pub name: String,
}

impl Member {
    pub async fn get(client: &Client, project_id: usize) -> anyhow::Result<Vec<Member>> {
        let res = client
            .get(&format!(
                "https://gitlab.inf.uni-konstanz.de/api/v4/projects/{}/members",
                project_id
            ))
            .send()
            .await?;

        Ok(res.json().await?)
    }
}
