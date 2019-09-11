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

//! Management of multiple windows.

use std::sync::atomic::{AtomicU32, Ordering};

use crate::kurbo::{Point, Rect, Size};

use crate::shell::window::WindowHandle;
use crate::{
    BoxConstraints, Data, Env, Event, EventCtx, LayoutCtx, LocalizedString, PaintCtx, UpdateCtx,
    Widget, WidgetPod,
};

/// A unique identifier for a window.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
// TODO: Remove Default when we get it fully wired up
#[derive(Default)]
pub struct WindowId(u32);

static WINDOW_ID_COUNTER: AtomicU32 = AtomicU32::new(1);

/// Per-window state not owned by user code.
pub struct Window<T: Data> {
    pub(crate) root: WidgetPod<T, Box<dyn Widget<T>>>,
    pub(crate) title: LocalizedString<T>,
    size: Size,
    // menu
    // delegate?
}

impl<T: Data> Window<T> {
    pub fn new(root: impl Widget<T> + 'static, title: LocalizedString<T>) -> Window<T> {
        Window {
            root: WidgetPod::new(Box::new(root)),
            size: Size::ZERO,
            title,
        }
    }

    pub fn event(
        &mut self,
        event: &Event,
        ctx: &mut EventCtx,
        data: &mut T,
        env: &Env,
        //window_id: WindowId,
    ) {
        if let Event::Size(size) = event {
            self.size = *size;
        }
        let _action = self.root.event(event, ctx, data, env);

        if let Some(cursor) = ctx.cursor {
            ctx.win_ctx.set_cursor(&cursor);
        }
    }

    pub fn update(&mut self, update_ctx: &mut UpdateCtx, data: &T, env: &Env) {
        self.update_title(&update_ctx.window, data, env);
        self.root.update(update_ctx, data, env);
    }

    pub fn layout(&mut self, layout_ctx: &mut LayoutCtx, data: &T, env: &Env) {
        let bc = BoxConstraints::tight(self.size);
        let size = self.root.layout(layout_ctx, &bc, data, env);
        self.root
            .set_layout_rect(Rect::from_origin_size(Point::ORIGIN, size));
    }

    pub fn paint(&mut self, paint_ctx: &mut PaintCtx, data: &T, env: &Env) {
        self.root.paint(paint_ctx, data, env);
    }

    pub(crate) fn update_title(&mut self, win_handle: &WindowHandle, data: &T, env: &Env) {
        if self.title.resolve(data, env) {
            win_handle.set_title(self.title.localized_str());
        }
    }
}

impl WindowId {
    /// Allocate a new, unique window id.
    ///
    /// Do note that if we create 4 billion windows there may be a collision.
    pub fn new() -> WindowId {
        let id = WINDOW_ID_COUNTER.fetch_add(1, Ordering::Relaxed);
        WindowId(id)
    }
}
