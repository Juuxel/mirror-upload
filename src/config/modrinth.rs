/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::modrinth::Dependency;
use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct ModrinthSettings {
    pub project_id: String,
    pub dependencies: Option<Vec<Dependency>>,
    pub version_number: Option<String>,
    pub slug: Option<String>,
}
