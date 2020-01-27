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

//! A widget that switches dynamically between two child views.

use crate::kurbo::{Point, Rect, Size};
use crate::{
    BoxConstraints, Data, Env, Event, EventCtx, LayoutCtx, LifeCycle, LifeCycleCtx, PaintCtx,
    UpdateCtx, Widget, WidgetPod,
};

/// A widget that switches between two possible child views.
pub struct Either<T: Data> {
    closure: Box<dyn Fn(&T, &Env) -> bool>,
    true_branch: WidgetPod<T, Box<dyn Widget<T>>>,
    false_branch: WidgetPod<T, Box<dyn Widget<T>>>,
    current: Option<bool>,
}

impl<T: Data> Either<T> {
    /// Create a new widget that switches between two views.
    ///
    /// The given closure is evaluated on data change. If its value is `true`, then
    /// the `true_branch` widget is shown, otherwise `false_branch`.
    pub fn new(
        closure: impl Fn(&T, &Env) -> bool + 'static,
        true_branch: impl Widget<T> + 'static,
        false_branch: impl Widget<T> + 'static,
    ) -> Either<T> {
        Either {
            closure: Box::new(closure),
            true_branch: WidgetPod::new(true_branch).boxed(),
            false_branch: WidgetPod::new(false_branch).boxed(),
            current: None,
        }
    }
}

impl<T: Data> Widget<T> for Either<T> {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut T, env: &Env) {
        if let Some(current) = self.current {
            if current {
                self.true_branch.event(ctx, event, data, env)
            } else {
                self.false_branch.event(ctx, event, data, env)
            }
        }
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &T, env: &Env) {
        self.true_branch.lifecycle(ctx, event, data, env);
        self.false_branch.lifecycle(ctx, event, data, env);
    }

    fn update(&mut self, ctx: &mut UpdateCtx, _old_data: &T, data: &T, env: &Env) {
        let new_current = (self.closure)(data, env);
        if self.current.is_none() {
            self.current = Some(new_current);
        }
        if let Some(current) = self.current {
            if new_current != current {
                self.current = Some(new_current);
                ctx.invalidate();
            }
            // TODO: more event flow to request here.
        }
        if new_current {
            self.true_branch.update(ctx, data, env);
        } else {
            self.false_branch.update(ctx, data, env);
        }
    }

    fn layout(
        &mut self,
        layout_ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        data: &T,
        env: &Env,
    ) -> Size {
        let current = self.current.unwrap_or((self.closure)(data, env));
        if current {
            let size = self.true_branch.layout(layout_ctx, bc, data, env);
            self.true_branch
                .set_layout_rect(Rect::from_origin_size(Point::ORIGIN, size));
            size
        } else {
            let size = self.false_branch.layout(layout_ctx, bc, data, env);
            self.false_branch
                .set_layout_rect(Rect::from_origin_size(Point::ORIGIN, size));
            size
        }
    }

    fn paint(&mut self, paint_ctx: &mut PaintCtx, data: &T, env: &Env) {
        let current = self.current.unwrap_or((self.closure)(data, env));
        if current {
            self.true_branch.paint(paint_ctx, data, env);
        } else {
            self.false_branch.paint(paint_ctx, data, env);
        }
    }
}
