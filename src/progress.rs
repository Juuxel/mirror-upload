/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use indicatif::{ProgressBar, ProgressStyle};

pub const SPINNER_CHARACTERS: &str = "-\\|/x";

pub fn simple_progress_bar_style() -> ProgressStyle {
    ProgressStyle::with_template("[{elapsed_precise}] {msg} {wide_bar} {pos}/{len}")
        .unwrap()
}

pub fn simple_progress_spinner_style() -> ProgressStyle {
    ProgressStyle::with_template("[{spinner}] {msg}")
        .unwrap()
        .tick_chars(SPINNER_CHARACTERS)
}

pub fn network_progress_bar(length: Option<u64>) -> ProgressBar {
    if let Some(length) = length {
        ProgressBar::new(length).with_style(network_progress_bar_style())
    } else {
        ProgressBar::new_spinner().with_style(network_progress_spinner_style())
    }
}

fn network_progress_bar_style() -> ProgressStyle {
    ProgressStyle::with_template("[{elapsed_precise}] {msg} {wide_bar} {bytes}/{total_bytes}")
        .unwrap()
}

fn network_progress_spinner_style() -> ProgressStyle {
    ProgressStyle::with_template("[{spinner}] {msg} {bytes}/{total_bytes} [{elapsed_precise}]")
        .unwrap()
        .tick_chars(SPINNER_CHARACTERS)
}
