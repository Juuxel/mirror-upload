/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

pub mod multipart;

use async_trait::async_trait;
use bytes::{BufMut, Bytes, BytesMut};
use futures::StreamExt;
use indicatif::MultiProgress;
use miette::{IntoDiagnostic, Result};
use reqwest::{Body, Client, Response};
use serde::de::DeserializeOwned;
use std::cmp::min;
use std::convert::Infallible;

pub use crate::config::Secrets;
use crate::error::MuError;
use crate::progress::network_progress_bar;

pub struct Context {
    pub client: Client,
    pub secrets: Secrets,
    pub progress: MultiProgress,
}

#[async_trait]
pub trait ApiRequest<T> {
    async fn request(&self, context: &Context) -> Result<T>;
}

pub async fn bytes_with_progress(context: &Context, response: Response) -> Result<Bytes> {
    let mut result = if let Some(len) = response.content_length() {
        BytesMut::with_capacity(len as usize)
    } else {
        BytesMut::new()
    };
    let bar = context
        .progress
        .add(network_progress_bar(response.content_length()));
    bar.set_message("Downloading...");

    let url = response.url().clone();
    let mut stream = response.bytes_stream();
    let mut downloaded: u64 = 0;

    while let Some(bytes) = stream.next().await {
        let bytes: Bytes = bytes.map_err(|err| {
            MuError::new(format!("Could not download {}", url))
                .cause(err)
                .to_report()
        })?;
        result.put_slice(&bytes);

        downloaded += bytes.len() as u64;
        if let Some(max) = bar.length() {
            downloaded = min(downloaded, max);
        }
        bar.set_position(downloaded);
    }

    bar.finish_and_clear();
    Ok(result.freeze())
}

pub async fn json_with_progress<T: DeserializeOwned>(
    context: &Context,
    response: Response,
) -> Result<T> {
    let bytes = bytes_with_progress(context, response).await?;
    serde_json::from_slice::<T>(&bytes).into_diagnostic()
}

const CHUNK_SIZE: usize = 8192;

pub fn body_with_progress(context: &Context, bytes: Bytes) -> Body {
    let progress_bar = context
        .progress
        .add(network_progress_bar(Some(bytes.len() as u64)));
    progress_bar.set_message("Uploading...");
    let stream = async_stream::stream! {
        let mut i: usize = 0;

        while i < bytes.len() {
            let start = i;
            let end = min(start + CHUNK_SIZE, bytes.len());
            i += end - start;
            yield Ok(bytes.slice(start..end)) as Result<Bytes, Infallible>;
            progress_bar.set_position(i as u64);
        }

        progress_bar.finish_and_clear();
    };
    Body::wrap_stream(stream)
}
