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

//! Demos basic list widget and list manipulations.

use std::sync::Arc;

use druid::piet::Color;

use druid::lens::{self, LensExt};
use druid::widget::{Button, DynLabel, Flex, List, Scroll, WidgetExt};
use druid::{AppLauncher, Data, Lens, LensWrap, Widget, WindowDesc};

#[derive(Clone, Data, Lens)]
struct AppData {
    left: Arc<Vec<u32>>,
    right: Arc<Vec<u32>>,
}

fn main() {
    let main_window = WindowDesc::new(ui_builder);
    // Set our initial data
    let data = AppData {
        left: Arc::new(vec![1, 2]),
        right: Arc::new(vec![1, 2, 3]),
    };
    AppLauncher::with_window(main_window)
        .use_simple_logger()
        .launch(data)
        .expect("launch failed");
}

fn ui_builder() -> impl Widget<AppData> {
    let mut root = Flex::column();

    // Build a button to add children to both lists
    root.add_child(
        Button::new("Add", |_, data: &mut AppData, _| {
            // Add child to left list
            let value = data.left.len() + 1;
            Arc::make_mut(&mut data.left).push(value as u32);

            // Add child to right list
            let value = data.right.len() + 1;
            Arc::make_mut(&mut data.right).push(value as u32);
        })
        .fix_height(30.0),
        0.0,
    );

    let mut lists = Flex::row();

    // Build a simple list
    lists.add_child(
        LensWrap::new(
            Scroll::new(List::new(|| {
                DynLabel::new(|item: &u32, _| format!("List item #{}", item))
                    .padding(10.0)
                    .expand()
                    .height(50.0)
                    .background(Color::rgb(0.5, 0.5, 0.5))
            }))
            .vertical(),
            AppData::left,
        ),
        1.0,
    );

    // Build a list with shared data
    lists.add_child(
        LensWrap::new(
            Scroll::new(List::new(|| {
                let mut row = Flex::row();
                row.add_child(
                    DynLabel::new(|(_, item): &(Arc<Vec<u32>>, u32), _| {
                        format!("List item #{}", item)
                    }),
                    1.0,
                );
                row.add_child(
                    Button::sized(
                        "Delete",
                        |_ctx, (shared, item): &mut (Arc<Vec<u32>>, u32), _env| {
                            // We have access to both child's data and shared data.
                            // Remove element from right list.
                            Arc::make_mut(shared).retain(|v| v != item);
                        },
                        80.0,
                        20.0,
                    ),
                    0.0,
                );

                row.padding(10.0)
                    .background(Color::rgb(0.5, 0.0, 0.5))
                    .fix_height(50.0)
            }))
            .vertical(),
            lens::Id.map(
                // Expose shared data with children data
                |d: &AppData| (d.right.clone(), d.right.clone()),
                |d, x| {
                    // If shared data was changed reflect the changes in our AppData
                    if !x.0.same(&d.right) {
                        d.right = x.0
                    }
                },
            ),
        ),
        1.0,
    );

    root.add_child(lists, 1.0);

    root
}
