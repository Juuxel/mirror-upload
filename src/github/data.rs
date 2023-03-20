/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use miette::{miette, Result};
use regex::Regex;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Release {
    pub tag_name: String,
    pub name: Option<String>,
    pub body: Option<String>,
    pub prerelease: bool,
    pub assets: Vec<Asset>,
}

impl Release {
    pub fn get_assets(&self, file_regex: &Option<Regex>) -> Vec<&Asset> {
        self.assets
            .iter()
            .filter(|asset| {
                if let Some(regex) = &file_regex {
                    regex.is_match(asset.name.as_str())
                } else {
                    true
                }
            })
            .collect()
    }
}

#[derive(Deserialize, Debug)]
pub struct Asset {
    pub url: String,
    pub name: String,
}

pub struct Repo {
    pub owner: String,
    pub name: String,
}

impl Repo {
    pub fn parse<T>(str: T) -> Result<Repo>
    where
        T: AsRef<str>,
    {
        let str = str.as_ref();
        let parts: Vec<&str> = str.splitn(2, '/').collect();

        if parts.len() != 2 {
            return Err(miette!(
                "Expected GitHub repository name in the format 'owner/repo', found {}",
                str
            ));
        }

        Ok(Repo {
            owner: parts[0].into(),
            name: parts[1].into(),
        })
    }
}
