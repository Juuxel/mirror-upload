/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum DependencyType {
    Required,
    Optional,
    Incompatible,
    Embedded
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Dependency {
    pub dependency_type: DependencyType,
    pub file_name: Option<String>,
    pub project_id: Option<String>,
    pub version_id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum VersionType {
    Release,
    Beta,
    Alpha
}
