/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::env::VarError;
use std::path::{Path, PathBuf};

use clap::Parser;
use miette::{miette, IntoDiagnostic, Result, WrapErr};
use reqwest::Client;
use tokio::fs::File;
use tokio::io::AsyncReadExt;

use mirror_upload::config::{Config, Project};
use mirror_upload::curseforge::upload_to_curseforge;
use mirror_upload::error::MuError;
use mirror_upload::github::{GetReleaseByTagName, Repo};
use mirror_upload::modrinth::upload_to_modrinth;
use mirror_upload::requests::{ApiRequest, Context, Secrets};

#[derive(Parser)]
#[command(version)]
struct Args {
    /// GitHub version tag
    version_tag: String,
    /// Config file (default: ./mirror_upload.config.toml)
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,
    /// Secrets file (default: ./mirror_upload.secrets.toml)
    #[arg(short, long, value_name = "FILE")]
    secrets: Option<PathBuf>,
    /// Use secrets from the GITHUB_TOKEN and CURSEFORGE_TOKEN environment variables.
    /// This also happens when the secrets file does not exist.
    #[arg(long)]
    env_secrets: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let client = Client::builder()
        .user_agent("Juuxel/mirror-upload")
        .build()
        .into_diagnostic()?;

    let args = Args::parse();
    let secrets = get_secrets(&args)
        .await
        .wrap_err("Could not find secrets")?;
    let config_path: PathBuf = args
        .config
        .unwrap_or(PathBuf::from("mirror_upload.config.toml"));
    let config: Config =
        toml::from_str(read_file(&config_path).await?.as_str()).into_diagnostic()?;

    let repo = Repo::parse(&config.github)?;
    let context = Context { client, secrets };
    let release = GetReleaseByTagName {
        owner: repo.owner,
        repo: repo.name,
        tag: args.version_tag,
    }
    .request(&context)
    .await?;
    println!("Found GitHub release");

    if release.assets.is_empty() {
        return Err(miette!("No assets in GitHub release!"));
    }

    let projects = if let Some(projects) = &config.projects {
        projects.clone()
    } else {
        vec![Project::empty()]
    };

    println!("Publishing {} projects", projects.len());

    for project in projects {
        if let Some(settings) = project.get_modrinth(&config) {
            upload_to_modrinth(&context, &config, &project, &release, settings).await?;
        }

        if let Some(settings) = project.get_curseforge(&config) {
            upload_to_curseforge(&context, &config, &project, &release, settings).await?;
        }
    }

    Ok(())
}

async fn get_secrets(args: &Args) -> Result<Secrets> {
    let secrets: Secrets = if let Some(path) = &args.secrets {
        if args.env_secrets {
            return Err(miette!(
                "Cannot set both -s and --env-secrets at the same time"
            ));
        } else if !path.as_path().exists() {
            let path_str = path.as_os_str().to_string_lossy();
            return Err(miette!("Secrets file {} does not exist", path_str));
        }

        let secrets_str = read_file(&path).await?;
        toml::from_str(secrets_str.as_str()).into_diagnostic()?
    } else {
        let path = PathBuf::from("mirror_upload.secrets.toml");

        if args.env_secrets || !path.as_path().exists() {
            Secrets {
                github_token: get_env("GITHUB_TOKEN")?
                    .ok_or_else(|| {
                        MuError::new("Missing environment variable GITHUB_TOKEN")
                            .help(if !args.env_secrets {
                                Some("Using environment variables because ./mirror_upload.secrets.toml doesn't exist")
                            } else {
                                None
                            })
                            .to_report()
                    })?,
                curseforge_token: get_env("CURSEFORGE_TOKEN")?,
            }
        } else {
            let secrets_str = read_file(&path).await?;
            toml::from_str(secrets_str.as_str()).into_diagnostic()?
        }
    };
    Ok(secrets)
}

fn get_env(key: &str) -> Result<Option<String>> {
    let result = std::env::var(key);
    match result {
        Ok(value) => Ok(Some(value)),
        Err(VarError::NotPresent) => Ok(None),
        Err(err) => Err(
            MuError::new(format!("Failed to get environment variable {}", key))
                .cause(err)
                .to_report(),
        ),
    }
}

async fn read_file<P>(path: P) -> Result<String>
where
    P: AsRef<Path>,
{
    let mut file = File::open(path).await.into_diagnostic()?;
    let mut result = String::new();
    file.read_to_string(&mut result).await.into_diagnostic()?;
    Ok(result)
}
