/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use async_trait::async_trait;
use miette::Result;
use reqwest::Client;

pub use crate::config::Secrets;

pub struct Context {
    pub client: Client,
    pub secrets: Secrets,
}

#[async_trait]
pub trait ApiRequest<T> {
    async fn request(&self, context: &Context) -> Result<T>;
}
