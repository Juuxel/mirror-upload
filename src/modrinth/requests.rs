/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use miette::{miette, IntoDiagnostic, Result, WrapErr};
use regex::Regex;
use reqwest::header::CONTENT_TYPE;
use serde::Serialize;

use crate::config::{Config, ModrinthSettings, Project, ReleaseLevel};
use crate::github::{Asset, GetAsset, Release};
use crate::modrinth::{Dependency, VersionType};
use crate::requests::multipart::Form;
use crate::requests::{body_with_progress, Context};
use crate::template::Template;

pub const API_URL: &str = "https://api.modrinth.com/v2";
pub const AUTH_KEY: &str = "Authorization";

#[derive(Serialize)]
pub struct CreateVersionData {
    pub name: String,
    pub version_number: String,
    pub changelog: Option<String>,
    pub dependencies: Vec<Dependency>,
    pub game_versions: Vec<String>,
    pub version_type: VersionType,
    pub loaders: Vec<String>,
    pub featured: bool,
    pub project_id: String,
    pub file_parts: Vec<String>,
    pub primary_file: String,
}

pub async fn upload_to_modrinth(
    context: &Context,
    config: &Config,
    project: &Project,
    release: &Release,
    settings: &ModrinthSettings,
) -> Result<()> {
    let mut form = Form::new();
    let file_regex: Option<Regex> = project.get_regex(config)?;
    let assets: Vec<&Asset> = release.get_assets(&file_regex);
    let file_parts: Vec<String> = assets.iter().map(|asset| asset.name.clone()).collect();

    let version_number = if let Some(template) = &settings.version_number {
        Template::parse(template)
            .and_then(|template| {
                template.resolve(|key| match key {
                    "tag" => Some(&release.tag_name),
                    _ => None,
                })
            })
            .wrap_err("Could not compute Modrinth version number")?
    } else {
        release.tag_name.clone()
    };

    let primary_file = file_parts.first().unwrap().to_string();
    let name = release.name.clone().unwrap_or(release.tag_name.clone());
    let data = CreateVersionData {
        name,
        version_number: version_number.clone(), // TODO: Take these by reference instead
        changelog: release.body.clone(),
        dependencies: settings.dependencies.clone().unwrap_or(vec![]),
        game_versions: project.get_game_versions(config)?,
        version_type: ReleaseLevel::get(config, release).as_modrinth(),
        loaders: project
            .get_loaders(config)?
            .iter()
            .map(|loader| loader.modrinth_id().to_string())
            .collect(),
        featured: false,
        project_id: settings.project_id.clone(),
        file_parts,
        primary_file,
    };
    form.text("data", serde_json::to_string(&data).into_diagnostic()?);

    for asset in assets {
        GetAsset(asset)
            .attach_to_form(context, &mut form, asset.name.clone())
            .await?;
    }

    let url = format!("{}/version", API_URL);
    let response = context
        .client
        .post(url)
        .header(AUTH_KEY, &context.secrets.github_token)
        .header(CONTENT_TYPE, form.content_type())
        .body(body_with_progress(context, form.bytes()))
        .send()
        .await
        .into_diagnostic()?;

    if !response.status().is_success() {
        return Err(miette!(
            "Could not upload project to Modrinth: {}\n{}",
            response.status(),
            response.text().await.into_diagnostic()?
        ));
    }

    // Print a link to the version. We can use the project ID
    // on Modrinth if the slug is missing.
    context
        .progress
        .println(format!(
            "{} https://modrinth.com/mod/{}/version/{}",
            console::style("Link:").bold().blue(),
            settings.slug.as_ref().unwrap_or(&settings.project_id),
            version_number
        ))
        .into_diagnostic()
        .wrap_err("Could not print link to release")?;

    Ok(())
}
