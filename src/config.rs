use crate::Error;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

const CONFIG_FILE_NAME: &str = "forked.yml";

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Manifest {
    pub projects: HashMap<String, Project>,
    pub projects_directory: PathBuf,
    pub templates_directory: PathBuf,
    pub feedbacks_directory: PathBuf,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Project {
    pub id: u32,
    pub members: Vec<Member>,
    pub repository: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Member {
    pub username: String,
    pub name: String,
}

impl Manifest {
    pub fn load() -> anyhow::Result<Self> {
        let data = fs::read_to_string(CONFIG_FILE_NAME)?;
        let config: Self = serde_yaml::from_str(&data)?;

        if let Some(project) = config
            .projects
            .values()
            .find(|project| project.members.is_empty())
        {
            return Err(Error::MissingUsername {
                project_id: project.id,
            }
            .into());
        }
        if let Some(project) = config
            .projects
            .values()
            .find(|project| project.repository.is_empty())
        {
            return Err(Error::MissingSsh {
                project_id: project.id,
            }
            .into());
        }

        Ok(config)
    }

    pub fn save(self) -> anyhow::Result<()> {
        fs::write(CONFIG_FILE_NAME, serde_yaml::to_string(&self)?)?;
        Ok(())
    }
}
