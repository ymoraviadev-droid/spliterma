use gio;
use gtk::prelude::*;
use gtk4 as gtk;

use crate::layout::persist::{load_layout, save_layout};

pub fn setup_global_menu(window: &gtk::ApplicationWindow) {
    let menubar = gio::Menu::new();
    let file_menu = gio::Menu::new();

    file_menu.append(Some("Save Layout"), Some("app.save-layout"));
    file_menu.append(Some("Load Layout"), Some("app.load-layout"));
    menubar.append_submenu(Some("File"), &file_menu);

    let app = window.application().unwrap();

    // Save: לוקח תמיד את ה-child הנוכחי של החלון
    let save_action = gio::SimpleAction::new("save-layout", None);
    let win_for_save = window.clone();
    save_action.connect_activate(move |_, _| {
        if let Some(child) = win_for_save.child() {
            save_layout(&child);
        }
    });
    app.add_action(&save_action);

    // Load
    let load_action = gio::SimpleAction::new("load-layout", None);
    let win_for_load = window.clone();
    load_action.connect_activate(move |_, _| {
        load_layout(&win_for_load);
    });
    app.add_action(&load_action);

    // קיצורים
    app.set_accels_for_action("app.save-layout", &["<Primary>S"]);
    app.set_accels_for_action("app.load-layout", &["<Primary>O"]);

    app.set_menubar(Some(&menubar));
}
