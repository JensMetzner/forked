// use anyhow::Context;
use clap::{ArgSettings, Clap};
use config::*;
use reqwest::{
    header::{HeaderMap, HeaderValue},
    Client,
};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::{thread, time};
use thiserror::Error;
use tinytemplate::TinyTemplate;

mod config;
mod json;

const DELAY: time::Duration = time::Duration::from_secs(10);

#[derive(Clap, Debug)]
#[clap(
    version = env!("CARGO_PKG_VERSION"),
    author = env!("CARGO_PKG_AUTHORS"),
    about = env!("CARGO_PKG_DESCRIPTION"),
)]
pub struct Opts {
    /// Id of the repository
    #[clap(short, env = "GITLAB_TOKEN", setting = ArgSettings::HideEnvValues)]
    gitlab_token: String,
    #[clap(subcommand)]
    pub subcmd: SubCommand,
}

#[derive(Clap, Debug)]
pub enum SubCommand {
    Init {
        /// Id of the root repository
        project_id: usize,
        /// Exclude members of forked projects (with username)
        #[clap(long)]
        exclude_members: Vec<String>,
        /// Path to projects directory
        #[clap(long)]
        projects_directory: Option<PathBuf>,
        /// Path to templates directory
        #[clap(long)]
        templates_directory: Option<PathBuf>,
        /// Path to feedbacks directory
        #[clap(long)]
        feedbacks_directory: Option<PathBuf>,
    },
    Clone,
    Pull,
    Checkout {
        /// Name of the branch
        branch: String,
    },
    Feedback {
        /// Name of the feedback template
        name: String,
    },
}

#[derive(Copy, Clone, Debug, Error)]
pub enum Error {
    #[error("`username` in project {} is empty", project_id)]
    MissingUsername { project_id: u32 },
    #[error("`repository` in project {} is empty", project_id)]
    MissingSsh { project_id: u32 },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let opts: Opts = Opts::parse();

    if let Err(e) = run(opts).await {
        if let Some(e) = e.downcast_ref::<Error>() {
            log::error!("{:?}", e);
            eprintln!("{} (exit code: 1)", &e);
            std::process::exit(1);
        } else {
            log::error!("{:?}", e);
            eprintln!("process didn't exit successfully: (exit code: 2)");
            std::process::exit(2);
        }
    }

    Ok(())
}

async fn run(opts: Opts) -> Result<(), anyhow::Error> {
    let mut headers = HeaderMap::new();
    headers.insert("PRIVATE-TOKEN", HeaderValue::from_str(&opts.gitlab_token)?);

    let client = Client::builder().default_headers(headers).build()?;

    match opts.subcmd {
        SubCommand::Init {
            project_id,
            exclude_members,
            projects_directory,
            templates_directory,
            feedbacks_directory,
        } => {
            let forks = json::Forks::get(&client, project_id).await?;

            let mut projects = HashMap::new();

            for fork in forks {
                thread::sleep(DELAY);

                let members: Vec<Member> = json::Member::get(&client, fork.id)
                    .await?
                    .into_iter()
                    .map(|member| Member {
                        username: member.username,
                        name: member.name,
                    })
                    .filter(|member| !exclude_members.contains(&member.username))
                    .collect();

                if members.is_empty() {
                    continue;
                }

                projects.insert(
                    fork.namespace.path,
                    Project {
                        id: fork.id as u32,
                        members,
                        repository: fork.ssh_url_to_repo,
                    },
                );
            }

            Manifest {
                projects,
                projects_directory: projects_directory.unwrap_or_else(|| "projects".into()),
                templates_directory: templates_directory.unwrap_or_else(|| "templates".into()),
                feedbacks_directory: feedbacks_directory.unwrap_or_else(|| "feedbacks".into()),
            }
            .save()
        }
        SubCommand::Clone => {
            let config = Manifest::load()?;
            fs::create_dir_all(&config.projects_directory)?;

            for (key, project) in config.projects {
                print!("running `git clone {} {}` ... ", &project.repository, &key);

                let output = Command::new("git")
                    .args(&["clone", &project.repository, &key])
                    .current_dir(&config.projects_directory)
                    .output()?;

                if output.status.success() {
                    println!("\u{2713}");
                } else {
                    println!("\u{2717}");
                }

                thread::sleep(DELAY);
            }
            Ok(())
        }
        SubCommand::Pull => {
            let config = Manifest::load()?;

            for key in config.projects.keys() {
                print!("running `git pull` for {} ... ", &key);

                let output = Command::new("git")
                    .args(&["pull"])
                    .current_dir(config.projects_directory.join(&key))
                    .output()?;

                if output.status.success() {
                    println!("\u{2713}");
                } else {
                    println!("\u{2717}");
                }

                thread::sleep(DELAY);
            }
            Ok(())
        }
        SubCommand::Checkout { branch } => {
            let config = Manifest::load()?;

            for key in config.projects.keys() {
                print!("running `git checkout {}` for {} ... ", branch, &key);

                let output = Command::new("git")
                    .args(&["checkout", &branch])
                    .current_dir(config.projects_directory.join(&key))
                    .output()?;

                if output.status.success() {
                    println!("\u{2713}");
                } else {
                    println!("\u{2717}");
                }
            }

            Ok(())
        }
        SubCommand::Feedback { name } => {
            let config = Manifest::load()?;

            let mut tt = TinyTemplate::new();
            let raw = fs::read_to_string(config.templates_directory.join(format!("{}.md", name)))?;
            tt.add_template("Feedback", raw.as_str())?;

            fs::create_dir_all(&config.feedbacks_directory.join(&name))?;

            for (key, project) in config.projects {
                let rendered = tt.render("Feedback", &project)?;
                fs::write(
                    config
                        .feedbacks_directory
                        .join(&name)
                        .join(format!("{}.md", key)),
                    rendered,
                )?;
            }
            Ok(())
        }
    }
}
