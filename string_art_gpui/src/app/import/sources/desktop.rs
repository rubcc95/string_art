//! Native image import: system file picker and files dropped from the OS.

use std::path::{Path, PathBuf};

use gpui::{
    AsyncApp, Context, Div, DragMoveEvent, Entity, ExternalPaths, FileDropEvent, Stateful,
    WeakEntity, Window, prelude::*,
};

use crate::app::AppView;
use crate::import::{self, ImportError};
use crate::ui::DragHover;

/// Nothing to install: the native picker is opened on demand and drops are
/// delivered by gpui itself.
pub fn init(_view: Entity<AppView>, _app: AsyncApp) {}

/// Opens the native file picker, filtered to the supported image extensions.
pub fn pick(window: &mut Window, cx: &mut Context<AppView>) {
    let dialog = rfd::AsyncFileDialog::new()
        .add_filter("Images", import::SUPPORTED_EXTENSIONS)
        .set_title("Select an image")
        .set_parent(&*window)
        .pick_file();

    cx.spawn(async move |this, cx| {
        let Some(file) = dialog.await else {
            return;
        };
        let path = file.path().to_path_buf();

        if matches!(import::extension_support(&path), Some(false)) {
            let message = import::unsupported_file_message(&display_name(&path));
            let _ = this.update(cx, |this, cx| this.import_error(message, cx));
            return;
        }

        read_and_send(path, this, cx).await;
    })
    .detach();
}

/// Adds the drag feedback and the file-drop handler to the application root.
pub fn attach_drop(root: Stateful<Div>, cx: &mut Context<AppView>) -> Stateful<Div> {
    root.on_drag_move::<ExternalPaths>(cx.listener(
        |this, event: &DragMoveEvent<ExternalPaths>, _, cx| {
            if !event.bounds.contains(&event.event.position) {
                this.set_drag_hover(None, cx);
                return;
            }
            let hover = drag_hover_for(event.drag(cx).paths(), this.is_busy());
            this.set_drag_hover(Some(hover), cx);
        },
    ))
    .on_file_drop_exit(cx.listener(|this, _: &FileDropEvent, _, cx| {
        this.set_drag_hover(None, cx);
    }))
    .on_drop::<ExternalPaths>(cx.listener(|this, paths: &ExternalPaths, _, cx| {
        this.set_drag_hover(None, cx);
        if this.is_busy() {
            return;
        }

        match import::select_drop_path(paths.paths()) {
            Some(path) => spawn_read(path.clone(), cx),
            None => {
                let name = dropped_name(paths);
                let message = import::unsupported_file_message(&name);
                this.import_error(message, cx);
            }
        }
    }))
}

/// Describes what will happen if the user releases the dragged files.
fn drag_hover_for(paths: &[PathBuf], busy: bool) -> DragHover {
    if busy {
        return DragHover::Busy;
    }

    match import::select_drop_path(paths) {
        Some(path) => DragHover::Accepted {
            name: Some(display_name(path)),
        },
        None => DragHover::Rejected {
            name: dropped_name_from(paths),
        },
    }
}

fn dropped_name(paths: &ExternalPaths) -> String {
    dropped_name_from(paths.paths())
}

fn dropped_name_from(paths: &[PathBuf]) -> String {
    paths
        .first()
        .map(|path| display_name(path))
        .unwrap_or_else(|| "The dropped file".to_string())
}

fn spawn_read(path: PathBuf, cx: &mut Context<AppView>) {
    cx.spawn(async move |this, cx| read_and_send(path, this, cx).await)
        .detach();
}

async fn read_and_send(path: PathBuf, this: WeakEntity<AppView>, cx: &mut AsyncApp) {
    let name = display_name(&path);
    let background = cx.background_executor().clone();
    let result =
        background
            .spawn(async move {
                std::fs::read(&path).map_err(|error| ImportError::Read(error.to_string()))
            })
            .await;

    match result {
        Ok(bytes) => {
            let _ = this.update(cx, |this, cx| this.import_bytes(name, bytes, cx));
        }
        Err(error) => {
            let _ = this.update(cx, |this, cx| this.import_error(error.to_string(), cx));
        }
    }
}

fn display_name(path: &Path) -> String {
    path.file_name()
        .and_then(|name| name.to_str())
        .map(str::to_string)
        .unwrap_or_else(|| path.display().to_string())
}
