// Copyright 2019 The xi-editor Authors.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Theme keys and initial values.

use crate::piet::Color;

use crate::{Env, Key};

pub const WINDOW_BACKGROUND_COLOR: Key<Color> = Key::new("window_background_color");

pub const LABEL_COLOR: Key<Color> = Key::new("label_color");

pub const PRIMARY_LIGHT: Key<Color> = Key::new("primary_light");
pub const PRIMARY_DARK: Key<Color> = Key::new("primary_dark");
pub const BACKGROUND_LIGHT: Key<Color> = Key::new("background_light");
pub const BACKGROUND_DARK: Key<Color> = Key::new("background_dark");
pub const FOREGROUND_LIGHT: Key<Color> = Key::new("foreground_light");
pub const FOREGROUND_DARK: Key<Color> = Key::new("foreground_dark");
pub const BUTTON_DARK: Key<Color> = Key::new("button_dark");
pub const BUTTON_LIGHT: Key<Color> = Key::new("button_light");
pub const BORDER: Key<Color> = Key::new("border");
pub const BORDER_LIGHT: Key<Color> = Key::new("border_light");
pub const SELECTION_COLOR: Key<Color> = Key::new("selection_color");
pub const CURSOR_COLOR: Key<Color> = Key::new("cursor_color");

pub const FONT_NAME: Key<&str> = Key::new("font_name");
pub const TEXT_SIZE_NORMAL: Key<f64> = Key::new("text_size_normal");
//TODO: what's a better name for these?
pub const HOW_TALL_THINGS_ARE: Key<f64> = Key::new("how_tall_things_are");
pub const TALLER_THINGS: Key<f64> = Key::new("taller_things");

/// An initial theme.
pub fn init() -> Env {
    let mut env = Env::default()
        .adding(WINDOW_BACKGROUND_COLOR, Color::rgb8(0x29, 0x29, 0x29))
        .adding(LABEL_COLOR, Color::rgb8(0xf0, 0xf0, 0xea))
        .adding(PRIMARY_LIGHT, Color::rgb8(0x5c, 0xc4, 0xff))
        .adding(PRIMARY_DARK, Color::rgb8(0x00, 0x8d, 0xdd))
        .adding(BACKGROUND_LIGHT, Color::rgb8(0x3a, 0x3a, 0x3a))
        .adding(BACKGROUND_DARK, Color::rgb8(0x31, 0x31, 0x31))
        .adding(FOREGROUND_LIGHT, Color::rgb8(0xf9, 0xf9, 0xf9))
        .adding(FOREGROUND_DARK, Color::rgb8(0xbf, 0xbf, 0xbf))
        .adding(BUTTON_DARK, Color::BLACK)
        .adding(BUTTON_LIGHT, Color::rgb8(0x21, 0x21, 0x21))
        .adding(BORDER, Color::rgb8(0x3a, 0x3a, 0x3a))
        .adding(BORDER_LIGHT, Color::rgb8(0xa1, 0xa1, 0xa1))
        .adding(SELECTION_COLOR, Color::rgb8(0xf3, 0x00, 0x21))
        .adding(CURSOR_COLOR, Color::WHITE)
        .adding(TEXT_SIZE_NORMAL, 15.0)
        .adding(HOW_TALL_THINGS_ARE, 18.0)
        .adding(TALLER_THINGS, 24.0);

    #[cfg(target_os = "windows")]
    {
        env = env.adding(FONT_NAME, "Segoe UI");
    }
    #[cfg(target_os = "macos")]
    {
        // Ideally this would be a reference to San Francisco, but Cairo's
        // "toy text" API doesn't seem to be able to access it easily.
        env = env.adding(FONT_NAME, "SF Pro Display Regular");
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        env = env.adding(FONT_NAME, "sans-serif");
    }
    env
}
