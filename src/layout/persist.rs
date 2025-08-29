use gtk::prelude::*;
use gtk4 as gtk;
use serde_json;

use crate::layout::extract::extract_layout;
use crate::layout::types::{SavedLayout, SplitType};
use crate::util::errors::show_error_dialog;

pub fn save_layout(root_widget: &gtk::Widget) {
    let dialog = gtk::FileChooserDialog::builder()
        .title("Save Layout")
        .action(gtk::FileChooserAction::Save)
        .modal(true)
        .build();

    dialog.add_button("Cancel", gtk::ResponseType::Cancel);
    dialog.add_button("Save", gtk::ResponseType::Accept);
    dialog.set_current_name("terminal_layout.json");

    if let Some(window) = root_widget
        .root()
        .and_then(|r| r.downcast::<gtk::Window>().ok())
    {
        dialog.set_transient_for(Some(&window));
    }

    let root_clone = root_widget.clone();
    dialog.connect_response(move |d, resp| {
        if resp == gtk::ResponseType::Accept {
            if let Some(file) = d.file() {
                if let Some(path) = file.path() {
                    match extract_layout(&root_clone) {
                        Ok(layout) => {
                            let saved = SavedLayout {
                                version: "1.0".into(),
                                root: layout,
                            };
                            match serde_json::to_string_pretty(&saved) {
                                Ok(json) => {
                                    if let Err(e) = std::fs::write(&path, json) {
                                        show_error_dialog("Failed to save layout", &e.to_string());
                                    } else {
                                        println!("Layout saved to: {}", path.display());
                                    }
                                }
                                Err(e) => show_error_dialog("Serialize error", &e.to_string()),
                            }
                        }
                        Err(e) => show_error_dialog("Extract layout failed", &e),
                    }
                }
            }
        }
        d.close();
    });

    dialog.present();
}

pub fn load_layout(window: &gtk::ApplicationWindow) {
    let dialog = gtk::FileChooserDialog::builder()
        .title("Load Layout")
        .action(gtk::FileChooserAction::Open)
        .modal(true)
        .build();

    dialog.add_button("Cancel", gtk::ResponseType::Cancel);
    dialog.add_button("Load", gtk::ResponseType::Accept);
    dialog.set_transient_for(Some(window));

    let filter = gtk::FileFilter::new();
    filter.add_pattern("*.json");
    filter.set_name(Some("JSON Layout Files"));
    dialog.add_filter(&filter);

    let win = window.clone();
    dialog.connect_response(move |d, resp| {
        if resp == gtk::ResponseType::Accept {
            if let Some(file) = d.file() {
                if let Some(path) = file.path() {
                    match std::fs::read_to_string(&path) {
                        Ok(json) => match serde_json::from_str::<SavedLayout>(&json) {
                            Ok(saved) => match build_layout_from_data(&saved.root) {
                                Ok(container) => {
                                    win.set_child(Some(&container));
                                    println!("Layout loaded from: {}", path.display());
                                }
                                Err(e) => show_error_dialog("Build layout failed", &e),
                            },
                            Err(e) => show_error_dialog("Parse error", &e.to_string()),
                        },
                        Err(e) => show_error_dialog("Read file failed", &e.to_string()),
                    }
                }
            }
        }
        d.close();
    });

    dialog.present();
}

pub fn build_layout_from_data(
    layout: &crate::layout::types::TerminalLayout,
) -> Result<gtk::Box, String> {
    use crate::ui::terminal::create_terminal_with_title;

    if layout.split_type.is_none() {
        let container =
            create_terminal_with_title(&layout.name, layout.color_index, Some(&layout.working_dir));
        Ok(container)
    } else {
        let children = &layout.children;
        if children.len() != 2 {
            return Err("Split must have exactly 2 children".into());
        }

        let orientation = match layout.split_type {
            Some(SplitType::Horizontal) => gtk::Orientation::Horizontal,
            Some(SplitType::Vertical) => gtk::Orientation::Vertical,
            None => unreachable!(),
        };

        let paned = gtk::Paned::new(orientation);
        paned.set_hexpand(true);
        paned.set_vexpand(true);
        paned.set_wide_handle(true);

        let start_child = build_layout_from_data(&children[0])?;
        let end_child = build_layout_from_data(&children[1])?;

        paned.set_start_child(Some(start_child.upcast_ref::<gtk::Widget>()));
        paned.set_end_child(Some(end_child.upcast_ref::<gtk::Widget>()));

        let wrapper = gtk::Box::new(gtk::Orientation::Vertical, 0);
        wrapper.set_hexpand(true);
        wrapper.set_vexpand(true);
        wrapper.append(&paned);

        Ok(wrapper)
    }
}
