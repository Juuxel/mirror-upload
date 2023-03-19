/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum ReleaseType {
    Release,
    Beta,
    Alpha
}

#[derive(Deserialize)]
pub struct GameVersionType {
    pub id: u32,
    pub slug: String,
}

#[derive(Deserialize)]
pub struct GameVersion {
    pub id: u32,
    pub name: String,
    #[serde(rename = "gameVersionTypeID")]
    pub game_version_type_id: u32,
}
