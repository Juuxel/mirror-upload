/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use miette::{miette, IntoDiagnostic, Result};
use regex::Regex;
use serde::Deserialize;

pub use curseforge::*;
pub use modrinth::*;

use crate::curseforge::ReleaseType;
use crate::github::Release;
use crate::modrinth::VersionType;

mod curseforge;
mod modrinth;

#[derive(Deserialize, Clone)]
pub struct Config {
    /// GitHub project (format: "owner/repo")
    pub github: String,
    /// Target loaders
    pub loaders: Option<Vec<Loader>>,
    /// CurseForge project ID
    pub curseforge: Option<CurseForgeSettings>,
    /// Modrinth configuration
    pub modrinth: Option<ModrinthSettings>,
    /// Projects
    pub projects: Option<Vec<Project>>,
    /// Game versions
    pub game_versions: Option<Vec<String>>,
    /// File regex
    pub file_regex: Option<String>,
    /// Release level
    pub release_level: Option<ReleaseLevel>,
}

#[derive(Deserialize, Clone)]
pub struct Project {
    /// Target loaders
    pub loaders: Option<Vec<Loader>>,
    /// CurseForge project ID
    pub curseforge: Option<CurseForgeSettings>,
    /// Modrinth configuration
    pub modrinth: Option<ModrinthSettings>,
    /// Game versions
    pub game_versions: Option<Vec<String>>,
    /// File regex
    pub file_regex: Option<String>,
}

impl Project {
    pub fn empty() -> Self {
        Project {
            loaders: None,
            curseforge: None,
            modrinth: None,
            game_versions: None,
            file_regex: None,
        }
    }

    pub fn get_regex(&self, config: &Config) -> Result<Option<Regex>> {
        let regex = if let Some(regex) = self.file_regex.clone().or(config.file_regex.clone()) {
            let regex = Regex::new(regex.as_str()).into_diagnostic()?;
            Some(regex)
        } else {
            None
        };
        Ok(regex)
    }

    pub fn get_game_versions(&self, config: &Config) -> Result<Vec<String>> {
        self.game_versions
            .clone()
            .or_else(|| config.game_versions.clone())
            .ok_or(miette!("No game versions defined!"))
    }

    pub fn get_loaders(&self, config: &Config) -> Result<Vec<Loader>> {
        self.loaders
            .clone()
            .or_else(|| config.loaders.clone())
            .ok_or(miette!("No loaders defined!"))
    }

    pub fn get_curseforge<'a>(&'a self, config: &'a Config) -> Option<&CurseForgeSettings> {
        self.curseforge.as_ref().or(config.curseforge.as_ref())
    }

    pub fn get_modrinth<'a>(&'a self, config: &'a Config) -> Option<&ModrinthSettings> {
        self.modrinth.as_ref().or(config.modrinth.as_ref())
    }
}

#[derive(Deserialize)]
pub struct Secrets {
    pub github_token: String,
    pub curseforge_token: String,
}

#[derive(Deserialize, Copy, Clone)]
#[serde(rename_all = "snake_case")]
pub enum ReleaseLevel {
    Release,
    Beta,
    Alpha,
}

impl ReleaseLevel {
    pub fn as_curseforge(&self) -> ReleaseType {
        match self {
            Self::Release => ReleaseType::Release,
            Self::Beta => ReleaseType::Beta,
            Self::Alpha => ReleaseType::Alpha,
        }
    }

    pub fn as_modrinth(&self) -> VersionType {
        match self {
            Self::Release => VersionType::Release,
            Self::Beta => VersionType::Beta,
            Self::Alpha => VersionType::Alpha,
        }
    }

    pub fn get(config: &Config, release: &Release) -> ReleaseLevel {
        if let Some(level) = &config.release_level {
            *level
        } else if release.prerelease {
            ReleaseLevel::Beta
        } else {
            ReleaseLevel::Release
        }
    }
}

#[derive(Deserialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum Loader {
    Fabric,
    Forge,
    Quilt,
}

impl Loader {
    pub fn values() -> Vec<Self> {
        vec![Self::Fabric, Self::Forge, Self::Quilt]
    }

    pub fn modrinth_id(&self) -> &'static str {
        match self {
            Self::Fabric => "fabric",
            Self::Forge => "forge",
            Self::Quilt => "quilt",
        }
    }

    pub fn curseforge_name(&self) -> &'static str {
        match self {
            Self::Fabric => "Fabric",
            Self::Forge => "Forge",
            Self::Quilt => "Quilt",
        }
    }
}
