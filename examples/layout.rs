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

//! This example shows how to construct a basic layout.

use druid::shell::piet::Color;
use druid::shell::{runloop, WindowBuilder};
use druid::widget::{Button, Column, Container, Label, Padding, Row, SizedBox};
use druid::{UiMain, UiState, Widget};

fn build_app() -> impl Widget<u32> {
    // Begin construction of vertical layout
    let mut col = Column::new();

    // Construct a horizontal layout.
    let mut header = Row::new();
    header.add_child(
        Container::new(SizedBox::new(Label::new("One")).width(60.0))
            .border(Color::WHITE, 3.0)
            .color(Color::rgb8(0x77, 0x77, 0))
            .padding(5.0),
        0.0,
    );
    // Spacing element that will fill all available space in between label
    // and a button. Notice that weight is non-zero.
    header.add_child(SizedBox::empty().expand(), 1.0);
    header.add_child(Padding::uniform(20.0, Button::new("Two")), 0.0);

    col.add_child(
        Container::new(SizedBox::new(header).height(100.0)).color(Color::rgb8(0, 0x77, 0x88)),
        0.0,
    );

    for i in 0..5 {
        // Give a larger weight to one of the buttons for it to
        // occupy more space.
        let weight = if i == 2 { 3.0 } else { 1.0 };
        col.add_child(Button::new(format!("Button #{}", i)), weight);
    }

    col
}

fn main() {
    druid::shell::init();

    let mut run_loop = runloop::RunLoop::new();
    let mut builder = WindowBuilder::new();

    // Build app layout
    let root = build_app();
    // Set up initial app state
    let state = UiState::new(root, 0u32);
    builder.set_title("Layout example");
    builder.set_handler(Box::new(UiMain::new(state)));

    let window = builder.build().unwrap();
    window.show();
    run_loop.run();
}
