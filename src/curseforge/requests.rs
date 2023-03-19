/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use async_trait::async_trait;
use miette::{IntoDiagnostic, miette, Result};
use reqwest::multipart::Form;
use serde::{Deserialize, Serialize};

use crate::config::{Config, Project, ReleaseLevel};
use crate::curseforge::{GameVersion, GameVersionType, ReleaseType};
use crate::github::{Asset, GetAsset, Release};
use crate::requests::{ApiRequest, Context};

const API_URL: &str = "https://minecraft.curseforge.com/api";
const AUTH_KEY: &str = "X-Api-Token";

pub struct GameVersionTypes;

#[async_trait]
impl ApiRequest<Vec<GameVersionType>> for GameVersionTypes {
    async fn request(&self, context: &Context) -> Result<Vec<GameVersionType>> {
        let url = format!("{}/game/version-types", API_URL);
        let response = context.client.get(url)
            .header(AUTH_KEY, &context.secrets.curseforge_token)
            .send()
            .await.into_diagnostic()?;

        if !response.status().is_success() {
            return Err(miette!("Could not get game version types from CurseForge: {}\n{}",
                response.status(), response.text().await.into_diagnostic()?));
        }

        response.json::<Vec<GameVersionType>>().await.into_diagnostic()
    }
}

pub struct GameVersions;

#[async_trait]
impl ApiRequest<Vec<GameVersion>> for GameVersions {
    async fn request(&self, context: &Context) -> Result<Vec<GameVersion>> {
        let url = format!("{}/game/versions", API_URL);
        let response = context.client.get(url)
            .header(AUTH_KEY, &context.secrets.curseforge_token)
            .send()
            .await.into_diagnostic()?;

        if !response.status().is_success() {
            return Err(miette!("Could not get game versions from CurseForge: {}\n{}",
                response.status(), response.text().await.into_diagnostic()?));
        }

        response.json::<Vec<GameVersion>>().await.into_diagnostic()
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectUploadFileData {
    pub changelog: String,
    pub changelog_type: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(rename = "parentFileID", skip_serializing_if = "Option::is_none")]
    pub parent_file_id: Option<u32>,
    pub game_versions: Vec<u32>,
    pub release_type: ReleaseType,
}

#[derive(Deserialize)]
pub struct ProjectUploadFileResponse {
    id: u32,
}

async fn upload_asset_to_curseforge(
    context: &Context,
    config: &Config,
    release: &Release,
    asset: &Asset,
    curseforge_id: &str,
    parent_file_id: Option<u32>,
    game_versions: &[u32],
) -> Result<ProjectUploadFileResponse> {
    let metadata = ProjectUploadFileData {
        changelog: release.body.clone().unwrap_or_default(),
        changelog_type: "markdown",
        display_name: release.name.clone(),
        parent_file_id,
        game_versions: Vec::from(game_versions),
        release_type: ReleaseLevel::get(config, release).as_curseforge(),
    };
    let mut form = Form::new()
        .text("metadata", serde_json::to_string(&metadata).into_diagnostic()?);
    form = GetAsset(asset)
        .attach_to_form(context, form, "file".to_string())
        .await?;

    let url = format!("{}/projects/{}/upload-file", API_URL, curseforge_id);
    let response = context.client.post(url)
        .header(AUTH_KEY, &context.secrets.curseforge_token)
        .multipart(form)
        .send()
        .await.into_diagnostic()?;

    if !response.status().is_success() {
        return Err(miette!("Could not upload file {:?} to CurseForge: {}\n{}",
                metadata.display_name, response.status(), response.text().await.into_diagnostic()?));
    }

    response.json::<ProjectUploadFileResponse>().await.into_diagnostic()
}

pub async fn upload_to_curseforge(
    context: &Context,
    config: &Config,
    project: &Project,
    release: &Release,
    curseforge_id: &str,
) -> Result<()> {
    println!("Uploading {} to CurseForge", release.tag_name);

    let allowed_game_version_types: Vec<u32> = GameVersionTypes.request(context).await?
        .iter()
        .filter(|version_type| {
            if version_type.slug.starts_with("minecraft-") {
                return true;
            }

            version_type.slug == "java" || version_type.slug == "modloader"
        })
        .map(|version_type| version_type.id)
        .collect();
    let mut game_versions = project.get_game_versions(config)?;

    for loader in project.get_loaders(config)? {
        game_versions.push(loader.curseforge_name().to_string());
    }

    let game_versions: Vec<u32> = GameVersions.request(context).await?
        .iter()
        .filter(|version| allowed_game_version_types.contains(&version.game_version_type_id))
        .filter(|version| game_versions.contains(&version.name.to_string()))
        .map(|version| version.id)
        .collect();

    let file_regex = project.get_regex(config)?;
    let assets = release.get_assets(&file_regex);

    let head = assets.first().unwrap();
    let tail: Vec<_> = assets.iter().skip(1).collect();
    let primary_id = upload_asset_to_curseforge(
        context,
        config,
        release,
        head,
        curseforge_id,
        None,
        &game_versions,
    ).await?.id;

    for asset in tail {
        upload_asset_to_curseforge(
            context,
            config,
            release,
            asset,
            curseforge_id,
            Some(primary_id),
            &game_versions,
        ).await?;
    }

    Ok(())
}
