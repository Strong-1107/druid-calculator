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

//! The fundamental druid types.

use std::collections::{HashMap, VecDeque};

use crate::bloom::Bloom;
use crate::kurbo::{Affine, Insets, Point, Rect, Shape, Size, Vec2};
use crate::piet::RenderContext;
use crate::{
    BoxConstraints, Command, Data, Env, Event, EventCtx, InternalEvent, InternalLifeCycle,
    LayoutCtx, LifeCycle, LifeCycleCtx, PaintCtx, Region, Target, TimerToken, UpdateCtx, Widget,
    WidgetId, WindowId,
};

/// Our queue type
pub(crate) type CommandQueue = VecDeque<(Target, Command)>;

/// A container for one widget in the hierarchy.
///
/// Generally, container widgets don't contain other widgets directly,
/// but rather contain a `WidgetPod`, which has additional state needed
/// for layout and for the widget to participate in event flow.
///
/// This struct also contains the previous data for a widget, which is
/// essential for the [`update`] method, both to decide when the update
/// needs to propagate, and to provide the previous data so that a
/// widget can process a diff between the old value and the new.
///
/// [`update`]: trait.Widget.html#tymethod.update
pub struct WidgetPod<T, W> {
    state: BaseState,
    old_data: Option<T>,
    env: Option<Env>,
    inner: W,
}

/// Generic state for all widgets in the hierarchy.
///
/// This struct contains the widget's layout rect, flags
/// indicating when the widget is active or focused, and other
/// state necessary for the widget to participate in event
/// flow.
///
/// It is provided to [`paint`] calls as a non-mutable reference,
/// largely so a widget can know its size, also because active
/// and focus state can affect the widget's appearance. Other than
/// that, widgets will generally not interact with it directly,
/// but it is an important part of the [`WidgetPod`] struct.
///
/// [`paint`]: trait.Widget.html#tymethod.paint
/// [`WidgetPod`]: struct.WidgetPod.html
#[derive(Clone)]
pub(crate) struct BaseState {
    pub(crate) id: WidgetId,
    /// The frame of this widget in its parents coordinate space.
    /// This should always be set; it is only an `Option` so that we
    /// can more easily track (and help debug) if it hasn't been set.
    layout_rect: Option<Rect>,
    /// The insets applied to the layout rect to generate the paint rect.
    /// In general, these will be zero; the exception is for things like
    /// drop shadows or overflowing text.
    pub(crate) paint_insets: Insets,

    // The region that needs to be repainted, relative to the widget's bounds.
    pub(crate) invalid: Region,

    // The part of this widget that is visible on the screen is offset by this
    // much. This will be non-zero for widgets that are children of `Scroll`, or
    // similar, and it is used for propagating invalid regions.
    pub(crate) viewport_offset: Vec2,

    // TODO: consider using bitflags for the booleans.
    pub(crate) is_hot: bool,

    pub(crate) is_active: bool,

    pub(crate) needs_layout: bool,

    /// Any descendant is active.
    has_active: bool,

    /// In the focused path, starting from window and ending at the focused widget.
    /// Descendants of the focused widget are not in the focused path.
    pub(crate) has_focus: bool,

    /// Any descendant has requested an animation frame.
    pub(crate) request_anim: bool,

    /// Any descendant has requested a timer.
    ///
    /// Note: we don't have any way of clearing this request, as it's
    /// likely not worth the complexity.
    pub(crate) request_timer: bool,

    pub(crate) focus_chain: Vec<WidgetId>,
    pub(crate) request_focus: Option<FocusChange>,
    pub(crate) children: Bloom<WidgetId>,
    pub(crate) children_changed: bool,
    /// Associate timers with widgets that requested them.
    pub(crate) timers: HashMap<TimerToken, WidgetId>,
}

/// Methods by which a widget can attempt to change focus state.
#[derive(Debug, Clone, Copy)]
pub(crate) enum FocusChange {
    /// The focused widget is giving up focus.
    Resign,
    /// A specific widget wants focus
    Focus(WidgetId),
    /// Focus should pass to the next focusable widget
    Next,
    /// Focus should pass to the previous focusable widget
    Previous,
}

impl<T, W: Widget<T>> WidgetPod<T, W> {
    /// Create a new widget pod.
    ///
    /// In a widget hierarchy, each widget is wrapped in a `WidgetPod`
    /// so it can participate in layout and event flow. The process of
    /// adding a child widget to a container should call this method.
    pub fn new(inner: W) -> WidgetPod<T, W> {
        let mut state = BaseState::new(inner.id().unwrap_or_else(WidgetId::next));
        state.children_changed = true;
        state.needs_layout = true;
        WidgetPod {
            state,
            old_data: None,
            env: None,
            inner,
        }
    }

    /// Read-only access to state. We don't mark the field as `pub` because
    /// we want to control mutation.
    pub(crate) fn state(&self) -> &BaseState {
        &self.state
    }

    /// Query the "active" state of the widget.
    pub fn is_active(&self) -> bool {
        self.state.is_active
    }

    /// Returns `true` if any descendant is active.
    pub fn has_active(&self) -> bool {
        self.state.has_active
    }

    /// Query the "hot" state of the widget.
    ///
    /// See [`EventCtx::is_hot`](struct.EventCtx.html#method.is_hot) for
    /// additional information.
    pub fn is_hot(&self) -> bool {
        self.state.is_hot
    }

    /// Return a reference to the inner widget.
    pub fn widget(&self) -> &W {
        &self.inner
    }

    /// Return a mutable reference to the inner widget.
    pub fn widget_mut(&mut self) -> &mut W {
        &mut self.inner
    }

    /// Get the identity of the widget.
    pub fn id(&self) -> WidgetId {
        self.state.id
    }

    /// Set layout rectangle.
    ///
    /// Intended to be called on child widget in container's `layout`
    /// implementation.
    pub fn set_layout_rect(&mut self, ctx: &mut LayoutCtx, data: &T, env: &Env, layout_rect: Rect) {
        self.state.layout_rect = Some(layout_rect);

        if WidgetPod::set_hot_state(
            &mut self.inner,
            ctx.command_queue,
            &mut self.state,
            ctx.window_id,
            layout_rect,
            ctx.mouse_pos,
            data,
            env,
        ) {
            ctx.base_state.merge_up(&self.state);
        }
    }

    #[deprecated(since = "0.5.0", note = "use layout_rect() instead")]
    #[doc(hidden)]
    pub fn get_layout_rect(&self) -> Rect {
        self.layout_rect()
    }

    /// The layout rectangle.
    ///
    /// This will be same value as set by `set_layout_rect`.
    pub fn layout_rect(&self) -> Rect {
        self.state.layout_rect.unwrap_or_default()
    }

    /// Set the viewport offset.
    ///
    /// This is relevant only for children of a scroll view (or similar). It must
    /// be set by the parent widget whenever it modifies the position of its child
    /// while painting it and propagating events. As a rule of thumb, you need this
    /// if and only if you `Affine::translate` the paint context before painting
    /// your child. For an example, see the implentation of [`Scroll`].
    ///
    /// [`Scroll`]: widget/struct.Scroll.html
    pub fn set_viewport_offset(&mut self, offset: Vec2) {
        self.state.viewport_offset = offset;
    }

    /// The viewport offset.
    ///
    /// This will be the same value as set by [`set_viewport_offset`].
    ///
    /// [`set_viewport_offset`]: #method.viewport_offset
    pub fn viewport_offset(&self) -> Vec2 {
        self.state.viewport_offset
    }

    /// Get the widget's paint [`Rect`].
    ///
    /// This is the [`Rect`] that widget has indicated it needs to paint in.
    /// This is the same as the [`layout_rect`] with the [`paint_insets`] applied;
    /// in the general case it is the same as the [`layout_rect`].
    ///
    /// [`layout_rect`]: #method.layout_rect
    /// [`Rect`]: struct.Rect.html
    /// [`paint_insets`]: #method.paint_insets
    pub fn paint_rect(&self) -> Rect {
        self.state.paint_rect()
    }

    /// Return the paint [`Insets`] for this widget.
    ///
    /// If these [`Insets`] are nonzero, they describe the area beyond a widget's
    /// layout rect where it needs to paint.
    ///
    /// These are generally zero; exceptions are widgets that do things like
    /// paint a drop shadow.
    ///
    /// A widget can set its insets by calling [`set_paint_insets`] during its
    /// [`layout`] method.
    ///
    /// [`Insets`]: struct.Insets.html
    /// [`set_paint_insets`]: struct.LayoutCtx.html#method.set_paint_insets
    /// [`layout`]: trait.Widget.html#tymethod.layout
    pub fn paint_insets(&self) -> Insets {
        self.state.paint_insets
    }

    /// Given a parents layout size, determine the appropriate paint `Insets`
    /// for the parent.
    ///
    /// This is a convenience method to be used from the [`layout`] method
    /// of a `Widget` that manages a child; it allows the parent to correctly
    /// propogate a child's desired paint rect, if it extends beyond the bounds
    /// of the parent's layout rect.
    ///
    /// [`layout`]: trait.Widget.html#tymethod.layout
    /// [`Insets`]: struct.Insets.html
    pub fn compute_parent_paint_insets(&self, parent_size: Size) -> Insets {
        let parent_bounds = Rect::ZERO.with_size(parent_size);
        let union_pant_rect = self.paint_rect().union(parent_bounds);
        union_pant_rect - parent_bounds
    }

    /// Determines if the provided `mouse_pos` is inside `rect`
    /// and if so updates the hot state and sends `LifeCycle::HotChanged`.
    ///
    /// Returns `true` if the hot state changed.
    ///
    /// The provided `child_state` should be merged up if this returns `true`.
    #[allow(clippy::too_many_arguments)]
    fn set_hot_state(
        child: &mut W,
        command_queue: &mut CommandQueue,
        child_state: &mut BaseState,
        window_id: WindowId,
        rect: Rect,
        mouse_pos: Option<Point>,
        data: &T,
        env: &Env,
    ) -> bool {
        let had_hot = child_state.is_hot;
        child_state.is_hot = match mouse_pos {
            Some(pos) => rect.winding(pos) != 0,
            None => false,
        };
        if had_hot != child_state.is_hot {
            let hot_changed_event = LifeCycle::HotChanged(child_state.is_hot);
            let mut child_ctx = LifeCycleCtx {
                command_queue,
                base_state: child_state,
                window_id,
            };
            child.lifecycle(&mut child_ctx, &hot_changed_event, data, env);
            return true;
        }
        false
    }
}

impl<T: Data, W: Widget<T>> WidgetPod<T, W> {
    /// Paint a child widget.
    ///
    /// Generally called by container widgets as part of their [`paint`]
    /// method.
    ///
    /// Note that this method does not apply the offset of the layout rect.
    /// If that is desired, use [`paint_with_offset`] instead.
    ///
    /// [`layout`]: trait.Widget.html#tymethod.layout
    /// [`paint`]: trait.Widget.html#tymethod.paint
    /// [`paint_with_offset`]: #method.paint_with_offset
    pub fn paint(&mut self, ctx: &mut PaintCtx, data: &T, env: &Env) {
        let mut inner_ctx = PaintCtx {
            render_ctx: ctx.render_ctx,
            window_id: ctx.window_id,
            z_ops: Vec::new(),
            region: ctx.region.clone(),
            base_state: &self.state,
            focus_widget: ctx.focus_widget,
        };
        self.inner.paint(&mut inner_ctx, data, env);
        ctx.z_ops.append(&mut inner_ctx.z_ops);

        if env.get(Env::DEBUG_PAINT) {
            const BORDER_WIDTH: f64 = 1.0;
            let rect = inner_ctx.size().to_rect().inset(BORDER_WIDTH / -2.0);
            let id = self.id().to_raw();
            let color = env.get_debug_color(id);
            inner_ctx.stroke(rect, &color, BORDER_WIDTH);
        }

        self.state.invalid = Region::EMPTY;
    }

    /// Paint the widget, translating it by the origin of its layout rectangle.
    ///
    /// This will recursively paint widgets, stopping if a widget's layout
    /// rect is outside of the currently visible region.
    // Discussion: should this be `paint` and the other `paint_raw`?
    pub fn paint_with_offset(&mut self, ctx: &mut PaintCtx, data: &T, env: &Env) {
        self.paint_with_offset_impl(ctx, data, env, false)
    }

    /// Paint the widget, even if its layout rect is outside of the currently
    /// visible region.
    pub fn paint_with_offset_always(&mut self, ctx: &mut PaintCtx, data: &T, env: &Env) {
        self.paint_with_offset_impl(ctx, data, env, true)
    }

    /// Shared implementation that can skip drawing non-visible content.
    fn paint_with_offset_impl(
        &mut self,
        ctx: &mut PaintCtx,
        data: &T,
        env: &Env,
        paint_if_not_visible: bool,
    ) {
        if !paint_if_not_visible && !ctx.region().intersects(self.state.paint_rect()) {
            return;
        }

        ctx.with_save(|ctx| {
            let layout_origin = self.layout_rect().origin().to_vec2();
            ctx.transform(Affine::translate(layout_origin));
            let visible = ctx.region().to_rect().intersect(self.state.paint_rect()) - layout_origin;
            ctx.with_child_ctx(visible, |ctx| self.paint(ctx, data, env));
        });
    }

    /// Compute layout of a widget.
    ///
    /// Generally called by container widgets as part of their [`layout`]
    /// method.
    ///
    /// [`layout`]: trait.Widget.html#tymethod.layout
    pub fn layout(
        &mut self,
        ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        data: &T,
        env: &Env,
    ) -> Size {
        self.state.needs_layout = false;

        let child_mouse_pos = match ctx.mouse_pos {
            Some(pos) => Some(pos - self.layout_rect().origin().to_vec2()),
            None => None,
        };
        let mut child_ctx = LayoutCtx {
            command_queue: ctx.command_queue,
            base_state: &mut self.state,
            window_id: ctx.window_id,
            text_factory: ctx.text_factory,
            mouse_pos: child_mouse_pos,
        };
        let size = self.inner.layout(&mut child_ctx, bc, data, env);

        ctx.base_state.merge_up(&child_ctx.base_state);

        if size.width.is_infinite() {
            let name = self.widget().type_name();
            log::warn!("Widget `{}` has an infinite width.", name);
        }
        if size.height.is_infinite() {
            let name = self.widget().type_name();
            log::warn!("Widget `{}` has an infinite height.", name);
        }
        size
    }

    /// Propagate an event.
    ///
    /// Generally the [`event`] method of a container widget will call this
    /// method on all its children. Here is where a great deal of the event
    /// flow logic resides, particularly whether to continue propagating
    /// the event.
    ///
    /// [`event`]: trait.Widget.html#tymethod.event
    pub fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut T, env: &Env) {
        if self.old_data.is_none() {
            log::error!(
                "widget {:?} is receiving an event without having first \
                 received WidgetAdded.",
                ctx.widget_id()
            );
        }

        // log if we seem not to be laid out when we should be
        if !matches!(event, Event::WindowConnected | Event::WindowSize(_))
            && self.state.layout_rect.is_none()
        {
            log::warn!(
                "Widget '{}' received an event ({:?}) without having been laid out. \
                This likely indicates a missed call to set_layout_rect.",
                self.inner.type_name(),
                event,
            );
        }

        // TODO: factor as much logic as possible into monomorphic functions.
        if ctx.is_handled {
            // This function is called by containers to propagate an event from
            // containers to children. Non-recurse events will be invoked directly
            // from other points in the library.
            return;
        }
        let had_active = self.state.has_active;
        let mut child_ctx = EventCtx {
            cursor: ctx.cursor,
            command_queue: ctx.command_queue,
            window: ctx.window,
            window_id: ctx.window_id,
            base_state: &mut self.state,
            is_handled: false,
            is_root: false,
            focus_widget: ctx.focus_widget,
        };

        let rect = child_ctx.base_state.layout_rect.unwrap_or_default();

        // Note: could also represent this as `Option<Event>`.
        let mut recurse = true;
        let child_event = match event {
            Event::Internal(internal) => match internal {
                InternalEvent::MouseLeave => {
                    let hot_changed = WidgetPod::set_hot_state(
                        &mut self.inner,
                        child_ctx.command_queue,
                        child_ctx.base_state,
                        child_ctx.window_id,
                        rect,
                        None,
                        data,
                        env,
                    );
                    recurse = had_active || hot_changed;
                    Event::Internal(InternalEvent::MouseLeave)
                }
                InternalEvent::TargetedCommand(target, cmd) => {
                    match target {
                        Target::Window(_) => Event::Command(cmd.clone()),
                        Target::Widget(id) if *id == child_ctx.widget_id() => {
                            Event::Command(cmd.clone())
                        }
                        Target::Widget(id) => {
                            // Recurse when the target widget could be our descendant.
                            // The bloom filter we're checking can return false positives.
                            recurse = child_ctx.base_state.children.may_contain(id);
                            Event::Internal(InternalEvent::TargetedCommand(*target, cmd.clone()))
                        }
                        Target::Global => {
                            panic!("Target::Global should be converted before WidgetPod")
                        }
                    }
                }
                InternalEvent::RouteTimer(token, widget_id) => {
                    let widget_id = *widget_id;
                    if widget_id != child_ctx.base_state.id {
                        recurse = child_ctx.base_state.children.may_contain(&widget_id);
                        Event::Internal(InternalEvent::RouteTimer(*token, widget_id))
                    } else {
                        Event::Timer(*token)
                    }
                }
            },
            Event::WindowConnected => Event::WindowConnected,
            Event::WindowSize(size) => {
                child_ctx.request_layout();
                recurse = ctx.is_root;
                Event::WindowSize(*size)
            }
            Event::MouseDown(mouse_event) => {
                WidgetPod::set_hot_state(
                    &mut self.inner,
                    child_ctx.command_queue,
                    child_ctx.base_state,
                    child_ctx.window_id,
                    rect,
                    Some(mouse_event.pos),
                    data,
                    env,
                );
                recurse = had_active || child_ctx.base_state.is_hot;
                let mut mouse_event = mouse_event.clone();
                mouse_event.pos -= rect.origin().to_vec2();
                Event::MouseDown(mouse_event)
            }
            Event::MouseUp(mouse_event) => {
                WidgetPod::set_hot_state(
                    &mut self.inner,
                    child_ctx.command_queue,
                    child_ctx.base_state,
                    child_ctx.window_id,
                    rect,
                    Some(mouse_event.pos),
                    data,
                    env,
                );
                recurse = had_active || child_ctx.base_state.is_hot;
                let mut mouse_event = mouse_event.clone();
                mouse_event.pos -= rect.origin().to_vec2();
                Event::MouseUp(mouse_event)
            }
            Event::MouseMove(mouse_event) => {
                let hot_changed = WidgetPod::set_hot_state(
                    &mut self.inner,
                    child_ctx.command_queue,
                    child_ctx.base_state,
                    child_ctx.window_id,
                    rect,
                    Some(mouse_event.pos),
                    data,
                    env,
                );
                recurse = had_active || child_ctx.base_state.is_hot || hot_changed;
                let mut mouse_event = mouse_event.clone();
                mouse_event.pos -= rect.origin().to_vec2();
                Event::MouseMove(mouse_event)
            }
            Event::KeyDown(e) => {
                recurse = child_ctx.has_focus();
                Event::KeyDown(*e)
            }
            Event::KeyUp(e) => {
                recurse = child_ctx.has_focus();
                Event::KeyUp(*e)
            }
            Event::Paste(e) => {
                recurse = child_ctx.has_focus();
                Event::Paste(e.clone())
            }
            Event::Wheel(wheel_event) => {
                recurse = had_active || child_ctx.base_state.is_hot;
                let mut wheel_event = wheel_event.clone();
                wheel_event.pos -= rect.origin().to_vec2();
                Event::Wheel(wheel_event)
            }
            Event::Zoom(zoom) => {
                recurse = had_active || child_ctx.base_state.is_hot;
                Event::Zoom(*zoom)
            }
            Event::Timer(token) => {
                recurse = false;
                Event::Timer(*token)
            }
            Event::Command(cmd) => Event::Command(cmd.clone()),
        };
        if recurse {
            child_ctx.base_state.has_active = false;
            self.inner.event(&mut child_ctx, &child_event, data, env);
            child_ctx.base_state.has_active |= child_ctx.base_state.is_active;
        };

        ctx.base_state.merge_up(&child_ctx.base_state);
        // Clear current widget's timers after merging with parent.
        child_ctx.base_state.timers.clear();
        ctx.is_handled |= child_ctx.is_handled;
    }

    pub fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &T, env: &Env) {
        // in the case of an internal routing event, if we are at our target
        // we may replace the routing event with the actual event
        let mut substitute_event = None;

        let recurse = match event {
            LifeCycle::Internal(internal) => match internal {
                InternalLifeCycle::RouteWidgetAdded => {
                    // if this is called either we were just created, in
                    // which case we need to change lifecycle event to
                    // WidgetAdded or in case we were already created
                    // we just pass this event down
                    if self.old_data.is_none() {
                        self.lifecycle(ctx, &LifeCycle::WidgetAdded, data, env);
                        return;
                    } else {
                        if self.state.children_changed {
                            self.state.children.clear();
                            self.state.focus_chain.clear();
                        }
                        self.state.children_changed
                    }
                }
                InternalLifeCycle::RouteFocusChanged { old, new } => {
                    self.state.request_focus = None;

                    let this_changed = if *old == Some(self.state.id) {
                        Some(false)
                    } else if *new == Some(self.state.id) {
                        Some(true)
                    } else {
                        None
                    };

                    if let Some(change) = this_changed {
                        // Only send FocusChanged in case there's actual change
                        if old != new {
                            self.state.has_focus = change;
                            substitute_event = Some(LifeCycle::FocusChanged(change));
                            true
                        } else {
                            false
                        }
                    } else {
                        self.state.has_focus = false;
                        // Recurse when the target widgets could be our descendants.
                        // The bloom filter we're checking can return false positives.
                        match (old, new) {
                            (Some(old), _) if self.state.children.may_contain(old) => true,
                            (_, Some(new)) if self.state.children.may_contain(new) => true,
                            _ => false,
                        }
                    }
                }
                #[cfg(test)]
                InternalLifeCycle::DebugRequestState { widget, state_cell } => {
                    if *widget == self.id() {
                        state_cell.set(self.state.clone());
                        false
                    } else {
                        // Recurse when the target widget could be our descendant.
                        // The bloom filter we're checking can return false positives.
                        self.state.children.may_contain(&widget)
                    }
                }
                #[cfg(test)]
                InternalLifeCycle::DebugInspectState(f) => {
                    f.call(&self.state);
                    true
                }
            },
            LifeCycle::AnimFrame(_) => {
                let r = self.state.request_anim;
                self.state.request_anim = false;
                r
            }
            LifeCycle::WidgetAdded => {
                assert!(self.old_data.is_none());

                self.old_data = Some(data.clone());
                self.env = Some(env.clone());

                true
            }
            LifeCycle::HotChanged(_) => false,
            LifeCycle::FocusChanged(_) => {
                // We are a descendant of a widget that has/had focus.
                // Descendants don't inherit focus, so don't recurse.
                false
            }
        };

        // use the substitute event, if one exists
        let event = substitute_event.as_ref().unwrap_or(event);

        if recurse {
            let mut child_ctx = LifeCycleCtx {
                command_queue: ctx.command_queue,
                base_state: &mut self.state,
                window_id: ctx.window_id,
            };
            self.inner.lifecycle(&mut child_ctx, event, data, env);
        }

        ctx.base_state.merge_up(&self.state);

        // we need to (re)register children in case of one of the following events
        match event {
            LifeCycle::WidgetAdded | LifeCycle::Internal(InternalLifeCycle::RouteWidgetAdded) => {
                self.state.children_changed = false;
                ctx.base_state.children = ctx.base_state.children.union(self.state.children);
                ctx.base_state.focus_chain.extend(&self.state.focus_chain);
                ctx.register_child(self.id());
            }
            _ => (),
        }
    }

    /// Propagate a data update.
    ///
    /// Generally called by container widgets as part of their [`update`]
    /// method.
    ///
    /// [`update`]: trait.Widget.html#tymethod.update
    pub fn update(&mut self, ctx: &mut UpdateCtx, data: &T, env: &Env) {
        match (self.old_data.as_ref(), self.env.as_ref()) {
            (Some(d), Some(e)) if d.same(data) && e.same(env) => return,
            (None, _) => {
                log::warn!("old_data missing in {:?}, skipping update", self.id());
                self.old_data = Some(data.clone());
                self.env = Some(env.clone());
                return;
            }
            _ => (),
        }

        let mut child_ctx = UpdateCtx {
            window: ctx.window,
            base_state: &mut self.state,
            window_id: ctx.window_id,
            command_queue: ctx.command_queue,
        };

        self.inner
            .update(&mut child_ctx, self.old_data.as_ref().unwrap(), data, env);
        self.old_data = Some(data.clone());
        self.env = Some(env.clone());

        ctx.base_state.merge_up(&self.state)
    }
}

impl<T, W: Widget<T> + 'static> WidgetPod<T, W> {
    /// Box the contained widget.
    ///
    /// Convert a `WidgetPod` containing a widget of a specific concrete type
    /// into a dynamically boxed widget.
    pub fn boxed(self) -> WidgetPod<T, Box<dyn Widget<T>>> {
        WidgetPod::new(Box::new(self.inner))
    }
}

impl BaseState {
    pub(crate) fn new(id: WidgetId) -> BaseState {
        BaseState {
            id,
            layout_rect: None,
            paint_insets: Insets::ZERO,
            invalid: Region::EMPTY,
            viewport_offset: Vec2::ZERO,
            is_hot: false,
            needs_layout: false,
            is_active: false,
            has_active: false,
            has_focus: false,
            request_anim: false,
            request_timer: false,
            request_focus: None,
            focus_chain: Vec::new(),
            children: Bloom::new(),
            children_changed: false,
            timers: HashMap::new(),
        }
    }

    pub(crate) fn add_timer(&mut self, timer_token: TimerToken) {
        self.timers.insert(timer_token, self.id);
    }

    /// Update to incorporate state changes from a child.
    fn merge_up(&mut self, child_state: &BaseState) {
        let mut child_region = child_state.invalid.clone();
        child_region += child_state.layout_rect().origin().to_vec2() - child_state.viewport_offset;
        let clip = self
            .layout_rect()
            .with_origin(Point::ORIGIN)
            .inset(self.paint_insets);
        child_region.intersect_with(clip);
        self.invalid.merge_with(child_region);

        self.needs_layout |= child_state.needs_layout;
        self.request_anim |= child_state.request_anim;
        self.request_timer |= child_state.request_timer;
        self.has_active |= child_state.has_active;
        self.has_focus |= child_state.has_focus;
        self.children_changed |= child_state.children_changed;
        self.request_focus = self.request_focus.or(child_state.request_focus);
        self.timers.extend(&child_state.timers);
    }

    #[inline]
    pub(crate) fn size(&self) -> Size {
        self.layout_rect.unwrap_or_default().size()
    }

    /// The paint region for this widget.
    ///
    /// For more information, see [`WidgetPod::paint_rect`].
    ///
    /// [`WidgetPod::paint_rect`]: struct.WidgetPod.html#method.paint_rect
    pub(crate) fn paint_rect(&self) -> Rect {
        self.layout_rect.unwrap_or_default() + self.paint_insets
    }

    pub(crate) fn layout_rect(&self) -> Rect {
        self.layout_rect.unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::widget::{Flex, Scroll, Split, TextBox};
    use crate::{WidgetExt, WindowId};

    const ID_1: WidgetId = WidgetId::reserved(0);
    const ID_2: WidgetId = WidgetId::reserved(1);
    const ID_3: WidgetId = WidgetId::reserved(2);

    #[test]
    fn register_children() {
        fn make_widgets() -> impl Widget<Option<u32>> {
            Split::columns(
                Flex::<Option<u32>>::row()
                    .with_child(TextBox::new().with_id(ID_1).parse())
                    .with_child(TextBox::new().with_id(ID_2).parse())
                    .with_child(TextBox::new().with_id(ID_3).parse()),
                Scroll::new(TextBox::new().parse()),
            )
        }

        let widget = make_widgets();
        let mut widget = WidgetPod::new(widget).boxed();

        let mut command_queue: CommandQueue = VecDeque::new();
        let mut state = BaseState::new(WidgetId::next());
        let mut ctx = LifeCycleCtx {
            command_queue: &mut command_queue,
            base_state: &mut state,
            window_id: WindowId::next(),
        };

        let env = Env::default();

        widget.lifecycle(&mut ctx, &LifeCycle::WidgetAdded, &None, &env);
        assert!(ctx.base_state.children.may_contain(&ID_1));
        assert!(ctx.base_state.children.may_contain(&ID_2));
        assert!(ctx.base_state.children.may_contain(&ID_3));
        assert_eq!(ctx.base_state.children.entry_count(), 7);
    }
}
