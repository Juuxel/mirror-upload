/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use async_trait::async_trait;
use miette::{Result};

pub use crate::config::Secrets;

#[async_trait]
pub trait ApiRequest<T> {
    async fn request(&self, client: &reqwest::Client, secrets: &Secrets) -> Result<T>;
}
