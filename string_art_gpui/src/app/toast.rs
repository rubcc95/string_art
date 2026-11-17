//! Global toast notifications.

use gpui::Context;

use crate::ui::{self, Toast, ToastEntry};

use super::AppView;

impl AppView {
    /// Shows a toast. It closes on its own after [`ui::TOAST_DURATION`] or when
    /// the user clicks it.
    pub fn toast(&mut self, toast: Toast, cx: &mut Context<Self>) {
        self.next_toast_id += 1;
        let id = self.next_toast_id;
        self.toasts.push(ToastEntry {
            id,
            toast,
            leaving: false,
        });
        cx.notify();

        cx.spawn(async move |this, cx| {
            cx.background_executor().timer(ui::TOAST_DURATION).await;
            let _ = this.update(cx, |this, cx| this.dismiss_toast(id, cx));
        })
        .detach();
    }

    /// Fades a toast out and removes it once the animation ends.
    pub fn dismiss_toast(&mut self, id: u64, cx: &mut Context<Self>) {
        let Some(entry) = self.toasts.iter_mut().find(|entry| entry.id == id) else {
            return;
        };
        if entry.leaving {
            return;
        }
        entry.leaving = true;
        cx.notify();

        cx.spawn(async move |this, cx| {
            cx.background_executor().timer(ui::TOAST_FADE).await;
            let _ = this.update(cx, |this, cx| {
                this.toasts
                    .retain(|entry| !(entry.id == id && entry.leaving));
                cx.notify();
            });
        })
        .detach();
    }
}
