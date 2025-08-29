use gtk::gio;
use gtk4 as gtk;
use vte4::prelude::*;

use crate::ui::{menus::setup_global_menu, terminal::create_terminal_with_title};

pub fn run() {
    let app = gtk::Application::builder()
        .application_id("com.spliterma.app")
        .build();

    app.connect_activate(|app| {
        // Prefer dark theme globally
        if let Some(settings) = gtk::Settings::default() {
            settings.set_gtk_application_prefer_dark_theme(true);
        }

        let window = gtk::ApplicationWindow::builder()
            .application(app)
            .title("Spliterma")
            .default_width(1000)
            .default_height(700)
            .build();

        // Set up application-level copy/paste actions BEFORE creating terminals
        setup_copy_paste_actions(app, &window);

        // טרמינל פתיחה
        let initial = create_terminal_with_title("Terminal 1", 0, None);

        // תפריט עליון + קיצורי מקלדת
        setup_global_menu(&window);

        window.set_child(Some(&initial));
        window.present();
    });

    app.run();
}

fn setup_copy_paste_actions(app: &gtk::Application, window: &gtk::ApplicationWindow) {
    // Copy action
    let copy_action = gio::SimpleAction::new("copy", None);
    let window_weak = window.downgrade();

    copy_action.connect_activate(move |_, _| {
        if let Some(window) = window_weak.upgrade() {
            if let Some(focused_terminal) = find_focused_terminal(&window) {
                if focused_terminal.has_selection() {
                    focused_terminal.copy_clipboard_format(vte4::Format::Text);
                } else {
                    println!("No selection to copy");
                }
            } else {
                println!("No focused terminal found for copy");
            }
        }
    });

    // Paste action
    let paste_action = gio::SimpleAction::new("paste", None);
    let window_weak2 = window.downgrade();

    paste_action.connect_activate(move |_, _| {
        if let Some(window) = window_weak2.upgrade() {
            if let Some(focused_terminal) = find_focused_terminal(&window) {
                focused_terminal.paste_clipboard();
            }
        }
    });

    // Add actions to application
    app.add_action(&copy_action);
    app.add_action(&paste_action);

    // Set keyboard accelerators - this is the key part for Flatpak!
    app.set_accels_for_action("app.copy", &["<Ctrl><Shift>c"]);
    app.set_accels_for_action("app.paste", &["<Ctrl><Shift>v"]);
}

fn find_focused_terminal(window: &gtk::ApplicationWindow) -> Option<vte4::Terminal> {
    // Method 1: Try to get the currently focused widget
    if let Some(focus_widget) = gtk::prelude::RootExt::focus(window) {
        // Check if it's a terminal directly
        if let Ok(terminal) = focus_widget.clone().downcast::<vte4::Terminal>() {
            return Some(terminal);
        }

        // Check if it's inside a terminal container
        let mut current = Some(focus_widget);
        while let Some(widget) = current {
            if let Ok(terminal) = widget.clone().downcast::<vte4::Terminal>() {
                return Some(terminal);
            }
            current = widget.parent();
        }
    }

    // Method 2: Search through all widgets to find terminals
    if let Some(root_child) = window.child() {
        if let Some(terminal) = find_terminal_recursive(&root_child) {
            return Some(terminal);
        }
    }

    println!("No terminal found");
    None
}

fn find_terminal_recursive(widget: &gtk::Widget) -> Option<vte4::Terminal> {
    // Check if this widget is a terminal
    if let Ok(terminal) = widget.clone().downcast::<vte4::Terminal>() {
        // Check if this terminal can receive focus or has focus
        if terminal.can_focus() {
            return Some(terminal);
        }
    }

    // Recursively check children
    let mut child = widget.first_child();
    while let Some(current_child) = child {
        if let Some(terminal) = find_terminal_recursive(&current_child) {
            return Some(terminal);
        }
        child = current_child.next_sibling();
    }

    None
}
