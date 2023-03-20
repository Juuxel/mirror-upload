/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use serde::Deserialize;
use crate::modrinth::Dependency;

#[derive(Deserialize, Clone)]
pub struct ModrinthSettings {
    pub project: String,
    pub dependencies: Option<Vec<Dependency>>,
}
