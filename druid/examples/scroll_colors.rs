// Copyright 2019 the Druid Authors
// SPDX-License-Identifier: Apache-2.0

//! This example allows to play with scroll bars over different color tones.

// On Windows platform, don't show a console when opening the app.
#![windows_subsystem = "windows"]

use druid::widget::{Container, Flex, Scroll, SizedBox};
use druid::{AppLauncher, Color, LocalizedString, Widget, WindowDesc};

fn build_app() -> impl Widget<u32> {
    let mut col = Flex::column();
    let rows = 30;
    let cols = 30;

    for i in 0..cols {
        let mut row = Flex::row();
        let col_progress = i as f64 / cols as f64;

        for j in 0..rows {
            let row_progress = j as f64 / rows as f64;

            row.add_child(
                Container::new(SizedBox::empty().width(200.0).height(200.0))
                    .background(Color::rgb(1.0 * col_progress, 1.0 * row_progress, 1.0)),
            );
        }

        col.add_child(row);
    }

    Scroll::new(col)
}

pub fn main() {
    let main_window = WindowDesc::new(build_app()).title(
        LocalizedString::new("scroll-colors-demo-window-title").with_placeholder("Rainbows!"),
    );
    let data = 0_u32;
    AppLauncher::with_window(main_window)
        .log_to_console()
        .launch(data)
        .expect("launch failed");
}
