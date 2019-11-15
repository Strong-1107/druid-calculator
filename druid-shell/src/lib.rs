// Copyright 2018 The xi-editor Authors.
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

//! Platform abstraction for druid toolkit.

#![deny(intra_doc_link_resolution_failure)]

pub use piet_common as piet;
pub use piet_common::kurbo;

#[cfg(target_os = "windows")]
#[macro_use]
extern crate winapi;

#[cfg(all(target_os = "macos", not(feature = "use_gtk")))]
#[macro_use]
extern crate objc;

#[cfg(not(any(feature = "use_gtk", target_os = "linux")))]
#[macro_use]
extern crate lazy_static;

pub mod clipboard;
mod common_util;
pub mod dialog;
pub mod error;
pub mod hotkey;
pub mod keyboard;
pub mod keycodes;
pub mod platform;
pub mod window;

pub use error::Error;
pub use platform::application;
pub use platform::menu;
pub use platform::util;
pub use platform::win_main as runloop; // TODO: rename to "runloop"
pub use platform::WindowBuilder;
pub use util::{get_locale, init};
