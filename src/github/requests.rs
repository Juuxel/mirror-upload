/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use async_trait::async_trait;
use miette::{miette, IntoDiagnostic, Result};
use reqwest::multipart::{Form, Part};

use crate::github::{Asset, Release};
use crate::requests::{ApiRequest, Context};

const API_URL: &str = "https://api.github.com";
const API_VERSION_KEY: &str = "X-GitHub-Api-Version";
const API_VERSION: &str = "2022-11-28";
const AUTH_KEY: &str = "Authorization";
const JSON_CONTENT_TYPE: &str = "application/vnd.github+json";

pub struct GetReleaseByTagName {
    pub owner: String,
    pub repo: String,
    pub tag: String,
}

#[async_trait]
impl ApiRequest<Release> for GetReleaseByTagName {
    async fn request(&self, context: &Context) -> Result<Release> {
        let url = format!(
            "{}/repos/{}/{}/releases/tags/{}",
            API_URL, self.owner, self.repo, self.tag
        );
        let response = context
            .client
            .get(url)
            .header("Accept", JSON_CONTENT_TYPE)
            .header(AUTH_KEY, &context.secrets.github_token)
            .header(API_VERSION_KEY, API_VERSION)
            .send()
            .await
            .into_diagnostic()?;

        if !response.status().is_success() {
            return Err(miette!(
                "Could not get release {}/{}@{} from GitHub: {}\n{}",
                self.owner,
                self.repo,
                self.tag,
                response.status(),
                response.text().await.into_diagnostic()?
            ));
        }

        response.json::<Release>().await.into_diagnostic()
    }
}

pub struct GetAsset<'a>(pub &'a Asset);

impl GetAsset<'_> {
    pub async fn attach_to_form(
        &self,
        context: &Context,
        form: Form,
        field_name: String,
    ) -> Result<Form> {
        println!("File: {}", self.0.name);
        let asset_bytes = self.request(context).await?;
        let part = Part::stream(asset_bytes).file_name(self.0.name.clone());
        Ok(form.part(field_name, part))
    }
}

#[async_trait]
impl ApiRequest<bytes::Bytes> for GetAsset<'_> {
    async fn request(&self, context: &Context) -> Result<bytes::Bytes> {
        let response = context
            .client
            .get(&self.0.url)
            .header("Accept", "application/octet-stream")
            .header(AUTH_KEY, &context.secrets.github_token)
            .header(API_VERSION_KEY, API_VERSION)
            .send()
            .await
            .into_diagnostic()?;

        if !response.status().is_success() {
            return Err(miette!(
                "Could not get asset file from GitHub at {}: {}\n{}",
                self.0.url,
                response.status(),
                response.text().await.into_diagnostic()?
            ));
        }

        response.bytes().await.into_diagnostic()
    }
}
