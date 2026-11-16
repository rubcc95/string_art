//! Window chrome wiring for the application title bar.

use gpui::{Context, Div, Stateful, Window, div, prelude::*, rgb};

use crate::ui;

use super::AppView;

impl AppView {
    /// Installs the observers that keep the window chrome in sync with the
    /// platform. Called once the window exists.
    pub fn install_window_observers(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.window_subscriptions
            .push(cx.observe_window_bounds(window, |_, _, cx| cx.notify()));
        self.window_subscriptions
            .push(cx.observe_button_layout_changed(window, |_, _, cx| cx.notify()));
        self.window_subscriptions
            .push(cx.observe_window_activation(window, |_, _, cx| cx.notify()));
    }

    /// The application title bar. It doubles as the window chrome: dragging it
    /// moves the window, double clicking maximizes or restores it, and it hosts
    /// the platform window controls.
    pub(super) fn title_bar(
        &self,
        trailing: impl IntoElement,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let drag_region = div()
            .id("title-bar-drag-region")
            .flex_1()
            .min_w_0()
            .h_full()
            .flex()
            .items_center()
            .pl_3()
            .text_lg()
            .child("String Art");

        // On Linux whether the application owns the chrome depends on the
        // decorations the compositor granted; on the other platforms this is
        // decided at compile time.
        let drag_region = if ui::title_bar::custom_chrome(window) {
            Self::platform_title_bar_behaviour(drag_region, window, cx)
        } else {
            drag_region
        };

        div()
            .id("title-bar")
            .flex_none()
            .h(ui::title_bar::HEIGHT)
            .flex()
            .items_center()
            .border_b_1()
            .border_color(rgb(ui::BORDER))
            .bg(rgb(ui::PANEL))
            .text_color(rgb(ui::TEXT))
            .pl(ui::title_bar::LEFT_INSET)
            .pr(ui::title_bar::RIGHT_INSET)
            .children(ui::title_bar::left_controls(window, cx))
            .child(drag_region)
            .child(trailing)
            .children(ui::title_bar::right_controls(window, cx))
    }

    /// Attaches the window-dragging behaviour. Clicking must not move the
    /// window, so the move only starts once the pointer actually moves; this
    /// also keeps double clicks and the buttons in the title bar working.
    #[cfg(any(target_os = "linux", target_os = "freebsd", target_os = "macos"))]
    fn attach_title_bar_drag(drag_region: Stateful<Div>, cx: &mut Context<Self>) -> Stateful<Div> {
        use gpui::MouseButton;

        drag_region
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, _, _, _| this.titlebar_should_move = true),
            )
            .on_mouse_up(
                MouseButton::Left,
                cx.listener(|this, _, _, _| this.titlebar_should_move = false),
            )
            .on_mouse_down_out(cx.listener(|this, _, _, _| this.titlebar_should_move = false))
            .on_mouse_move(cx.listener(|this, _, window, _| {
                if this.titlebar_should_move {
                    this.titlebar_should_move = false;
                    window.start_window_move();
                }
            }))
    }

    #[cfg(any(target_os = "linux", target_os = "freebsd"))]
    fn platform_title_bar_behaviour(
        drag_region: Stateful<Div>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        use gpui::MouseButton;

        let drag_region = Self::attach_title_bar_drag(drag_region, cx).on_click(cx.listener(
            |_, event: &gpui::ClickEvent, window, _| {
                if event.click_count() == 2 && window.is_resizable() {
                    window.zoom_window();
                }
            },
        ));

        if window.window_controls().window_menu {
            drag_region.on_mouse_down(MouseButton::Right, |event, window, _| {
                window.show_window_menu(event.position);
            })
        } else {
            drag_region
        }
    }

    #[cfg(target_os = "macos")]
    fn platform_title_bar_behaviour(
        drag_region: Stateful<Div>,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        Self::attach_title_bar_drag(drag_region, cx).on_click(cx.listener(
            |_, event: &gpui::ClickEvent, window, _| {
                if event.click_count() == 2 {
                    window.titlebar_double_click();
                }
            },
        ))
    }

    /// Windows and the web build have no app-owned dragging. On Windows the
    /// `Drag` control area makes the platform treat the region as the caption.
    #[cfg(not(any(target_os = "linux", target_os = "freebsd", target_os = "macos")))]
    fn platform_title_bar_behaviour(
        drag_region: Stateful<Div>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        #[cfg(target_os = "windows")]
        {
            drag_region.window_control_area(gpui::WindowControlArea::Drag)
        }
        #[cfg(not(target_os = "windows"))]
        {
            drag_region
        }
    }
}
