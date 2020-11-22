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
    /// Personal access token. (Not necessary if the environment variable `GITLAB_TOKEN` is set)
    #[clap(short='g', long, env = "GITLAB_TOKEN", setting = ArgSettings::HideEnvValues)]
    gitlab_token: String,
    /// Gitlab api url. (Not necessary if the environment variable `GITLAB_API` is set)
    #[clap(short='a', long, env = "GITLAB_API", setting = ArgSettings::HideEnvValues)]
    gitlab_api_url: String,
    #[clap(subcommand)]
    pub subcmd: SubCommand,
}

#[derive(Clap, Debug)]
pub enum FeedbackAction {
    Create,
    Publish,
}

#[derive(Clap, Debug)]
pub enum SubCommand {
    /// Initialize a course, adding all forked repositories to `forked.yml`
    Init {
        /// Id of the root repository
        project_id: u32,
        /// Exclude members of forked projects (with username)
        #[clap(long)]
        exclude_members: Vec<String>,
        /// Path to projects directory
        #[clap(long, default_value = "projects")]
        projects_directory: PathBuf,
        /// Path to templates directory
        #[clap(long, default_value = "templates")]
        templates_directory: PathBuf,
        /// Path to feedbacks directory
        #[clap(long, default_value = "feedbacks")]
        feedbacks_directory: PathBuf,
    },
    /// Runs `git clone <repository>` for all groups
    Clone,
    /// Runs `git pull` for all groups
    Pull,
    /// Runs `git checkout <branch>` for all groups
    Checkout {
        /// Name of the branch
        branch: String,
    },
    /// Either create or publish all feedback files for all groups
    Feedback {
        /// Choose the action
        #[clap(arg_enum)]
        action: FeedbackAction,
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
            let forks = json::Forks::get(&client, &opts.gitlab_api_url, project_id).await?;

            thread::sleep(DELAY);

            let mut projects = HashMap::new();

            for fork in forks {
                let members: Vec<Member> =
                    json::Member::get(&client, &opts.gitlab_api_url, fork.id)
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

                thread::sleep(DELAY);
            }

            Manifest {
                projects,
                projects_directory,
                templates_directory,
                feedbacks_directory,
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
        SubCommand::Feedback { name, action } => {
            let config = Manifest::load()?;

            match action {
                FeedbackAction::Create => {
                    let mut tt = TinyTemplate::new();
                    let raw = fs::read_to_string(
                        config.templates_directory.join(format!("{}.md", name)),
                    )?;
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
                }
                FeedbackAction::Publish => {
                    for (key, project) in config.projects {
                        print!("publishing issue for {} ... ", &key);

                        let feedback_path = config
                            .feedbacks_directory
                            .join(&name)
                            .join(format!("{}.md", key));

                        let data = fs::read_to_string(feedback_path)?;

                        let mut lines = data.lines();

                        let title = lines
                            .next()
                            .unwrap_or_else(|| "Feedback")
                            .trim_matches('#')
                            .trim()
                            .to_string();

                        let request = json::NewIssueRequest {
                            title,
                            description: lines.collect::<Vec<_>>().join("\n"),
                            labels: vec!["feedback".into()],
                        };

                        if request
                            .post(&client, &opts.gitlab_api_url, project.id)
                            .await?
                            .is_opened()
                        {
                            println!("\u{2713}");
                        } else {
                            println!("\u{2717}");
                        }
                        thread::sleep(DELAY);
                    }
                }
            }
            Ok(())
        }
    }
}
