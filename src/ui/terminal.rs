use gtk4 as gtk;
use gtk::{gio, glib};                // בשביל gio::Menu / glib::SpawnFlags
use vte4::{Terminal, PtyFlags};
use vte4::prelude::*;                // <<< מביא TerminalExtManual (spawn_async)
use crate::constants::TERMINAL_COLORS;
use crate::layout::persist::save_layout;
use crate::ui::split::{split_terminal, stop_terminal};

fn create_terminal_with_working_dir(working_dir: Option<&str>) -> Terminal {
    let terminal = Terminal::new();
    
    // Make terminal expand to fill available space
    terminal.set_hexpand(true);
    terminal.set_vexpand(true);
    
    // Spawn bash in the terminal with specific working directory
    let workdir = working_dir.map(|s| s.to_string());
    terminal.spawn_async(
        PtyFlags::DEFAULT,
        workdir.as_deref(),
        &["/bin/bash"],
        &[],
        glib::SpawnFlags::DEFAULT,
        || {},
        -1,
        None::<&gio::Cancellable>,
        |result| {
            if let Err(err) = result {
                eprintln!("spawn failed: {err}");
            }
        },
    );
    
    terminal
}

fn _create_terminal() -> Terminal {
    create_terminal_with_working_dir(None)
}

pub(crate) fn create_terminal_with_title(title: &str, color_index: usize, working_dir: Option<&str>) -> gtk::Box {
    // Create container for title + terminal
    let container = gtk::Box::new(gtk::Orientation::Vertical, 0);
    container.set_hexpand(true);
    container.set_vexpand(true);
    
    // Create title bar
    let title_bar = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    title_bar.set_margin_start(8);
    title_bar.set_margin_end(8);
    title_bar.set_margin_top(4);
    title_bar.set_margin_bottom(4);
    
    // Set title bar background color
    let color = TERMINAL_COLORS[color_index % TERMINAL_COLORS.len()];
    title_bar.set_css_classes(&["terminal-title"]);
    
    let css_provider = gtk::CssProvider::new();
    css_provider.load_from_data(&format!(
        ".terminal-title {{ background-color: {}; border-radius: 6px; min-height: 30px; }}",
        color
    ));
    
    let style_context = title_bar.style_context();
    style_context.add_provider(&css_provider, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);
    
    // Create color picker button
    let color_button = gtk::Button::new();
    color_button.set_css_classes(&["flat"]);
    
    let color_icon = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    color_icon.set_size_request(16, 16);
    color_icon.set_css_classes(&["color-dot"]);
    
    let color_css = gtk::CssProvider::new();
    color_css.load_from_data(&format!(
        ".color-dot {{ background-color: {}; border-radius: 50%; min-width: 16px; min-height: 16px; border: 2px solid white; }}",
        color
    ));
    color_icon.style_context().add_provider(&color_css, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);
    
    color_button.set_child(Some(&color_icon));
    
    // Create editable title label
    let title_label = gtk::Label::new(Some(title));
    title_label.set_hexpand(true);
    title_label.set_halign(gtk::Align::Start);
    title_label.set_css_classes(&["terminal-title-text"]);
    
    let title_css = gtk::CssProvider::new();
    title_css.load_from_data(".terminal-title-text { color: white; font-weight: bold; padding: 4px; }");
    title_label.style_context().add_provider(&title_css, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);
    
    // Add double-click to edit title
    let title_gesture = gtk::GestureClick::new();
    title_gesture.set_button(1); // Left click
    
    let title_label_clone = title_label.clone();
    let title_bar_clone = title_bar.clone();
    title_gesture.connect_pressed(move |_gesture, n_press, _x, _y| {
        if n_press == 2 { // Double click
            show_rename_dialog(&title_label_clone, &title_bar_clone);
        }
    });
    title_label.add_controller(title_gesture);
    
    title_bar.append(&color_button);
    title_bar.append(&title_label);
    
    // Create terminal
    let terminal = create_terminal_with_working_dir(working_dir);
    
    // Store working directory in terminal's data for later retrieval
    unsafe {
        if let Some(dir) = working_dir {
            terminal.set_data("working_dir", dir.to_string());
        } else {
            terminal.set_data("working_dir", std::env::current_dir().unwrap_or_default().to_string_lossy().to_string());
        }
    }
    
    // Set up color picker popup
    setup_color_picker(&color_button, &title_bar, &color_icon);
    
    // Set up context menu for the terminal
    setup_context_menu(&terminal, &container);
    
    container.append(&title_bar);
    container.append(&terminal);
    
    container
}

fn setup_color_picker(color_button: &gtk::Button, title_bar: &gtk::Box, color_icon: &gtk::Box) {
    let popover = gtk::Popover::new();
    popover.set_parent(color_button);
    
    let color_grid = gtk::Grid::new();
    color_grid.set_column_spacing(8);
    color_grid.set_row_spacing(8);
    color_grid.set_margin_start(12);
    color_grid.set_margin_end(12);
    color_grid.set_margin_top(12);
    color_grid.set_margin_bottom(12);
    
    for (i, &color) in TERMINAL_COLORS.iter().enumerate() {
        let color_btn = gtk::Button::new();
        color_btn.set_size_request(24, 24);
        
        let color_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        color_box.set_css_classes(&["color-picker-dot"]);
        
        let picker_css = gtk::CssProvider::new();
        picker_css.load_from_data(&format!(
            ".color-picker-dot {{ background-color: {}; border-radius: 50%; min-width: 20px; min-height: 20px; border: 2px solid #ccc; }}",
            color
        ));
        color_box.style_context().add_provider(&picker_css, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);
        
        color_btn.set_child(Some(&color_box));
        color_btn.set_css_classes(&["flat"]);
        
        let title_bar_clone = title_bar.clone();
        let color_icon_clone = color_icon.clone();
        let popover_clone = popover.clone();
        let color_str = color.to_string();
        
        color_btn.connect_clicked(move |_| {
            // Update title bar background
            let css_provider = gtk::CssProvider::new();
            css_provider.load_from_data(&format!(
                ".terminal-title {{ background-color: {}; border-radius: 6px; min-height: 30px; }}",
                color_str
            ));
            title_bar_clone.style_context().add_provider(&css_provider, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);
            
            // Update color icon
            let color_css = gtk::CssProvider::new();
            color_css.load_from_data(&format!(
                ".color-dot {{ background-color: {}; border-radius: 50%; min-width: 16px; min-height: 16px; border: 2px solid white; }}",
                color_str
            ));
            color_icon_clone.style_context().add_provider(&color_css, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);
            
            popover_clone.popdown();
        });
        
        color_grid.attach(&color_btn, (i % 4) as i32, (i / 4) as i32, 1, 1);
    }
    
    popover.set_child(Some(&color_grid));
    
    color_button.connect_clicked(move |_| {
        popover.popup();
    });
}

fn show_rename_dialog(title_label: &gtk::Label, title_bar: &gtk::Box) {
    let dialog = gtk::Dialog::builder()
        .title("Rename Terminal")
        .modal(true)
        .build();
    
    if let Some(window) = title_bar.root().and_then(|root| root.downcast::<gtk::Window>().ok()) {
        dialog.set_transient_for(Some(&window));
    }
    
    let content_area = dialog.content_area();
    let entry = gtk::Entry::new();
    entry.set_text(&title_label.text());
    entry.set_margin_start(12);
    entry.set_margin_end(12);
    entry.set_margin_top(12);
    entry.set_margin_bottom(12);
    
    content_area.append(&entry);
    
    dialog.add_button("Cancel", gtk::ResponseType::Cancel);
    dialog.add_button("OK", gtk::ResponseType::Ok);
    dialog.set_default_response(gtk::ResponseType::Ok);
    
    let title_label_clone = title_label.clone();
    let entry_clone = entry.clone();
    let _dialog_clone = dialog.clone();
    dialog.connect_response(move |dialog, response| {
        if response == gtk::ResponseType::Ok {
            let new_text = entry_clone.text();
            if !new_text.is_empty() {
                title_label_clone.set_text(&new_text);
            }
        }
        dialog.close();
    });
    
    let dialog_clone2 = dialog.clone();
    entry.connect_activate(move |_| {
        dialog_clone2.response(gtk::ResponseType::Ok);
    });
    
    dialog.present();
    entry.grab_focus();
}

fn setup_context_menu(terminal: &Terminal, container: &gtk::Box) {
    // Create the context menu
    let menu = gio::Menu::new();
    menu.append(Some("Split Horizontal"), Some("split.horizontal"));
    menu.append(Some("Split Vertical"), Some("split.vertical"));
    menu.append(Some("Save Layout"), Some("terminal.save-layout"));
    menu.append(Some("Stop Terminal"), Some("terminal.stop"));
    
    let popover_menu = gtk::PopoverMenu::from_model(Some(&menu));
    popover_menu.set_parent(terminal);
    
    // Create action group for split actions
    let action_group = gio::SimpleActionGroup::new();
    
    // Clone references for the closures
    let _terminal_clone = terminal.clone();
    let container_clone = container.clone();
    let popover_clone = popover_menu.clone();
    
    // Horizontal split action
    let horizontal_action = gio::SimpleAction::new("horizontal", None);
    horizontal_action.connect_activate(move |_, _| {
        split_terminal(&container_clone, gtk::Orientation::Horizontal);
        popover_clone.popdown();
    });
    action_group.add_action(&horizontal_action);
    
    // Vertical split action  
    let container_clone2 = container.clone();
    let popover_clone2 = popover_menu.clone();
    
    let vertical_action = gio::SimpleAction::new("vertical", None);
    vertical_action.connect_activate(move |_, _| {
        split_terminal(&container_clone2, gtk::Orientation::Vertical);
        popover_clone2.popdown();
    });
    action_group.add_action(&vertical_action);
    
    // Save layout action
    let container_clone4 = container.clone();
    let popover_clone4 = popover_menu.clone();
    
    let save_layout_action = gio::SimpleAction::new("save-layout", None);
    save_layout_action.connect_activate(move |_, _| {
        save_layout(container_clone4.upcast_ref::<gtk::Widget>());
        popover_clone4.popdown();
    });
    action_group.add_action(&save_layout_action);
    
    // Stop terminal action
    let terminal_clone2 = terminal.clone();
    let container_clone3 = container.clone();
    let popover_clone3 = popover_menu.clone();
    
    let stop_action = gio::SimpleAction::new("stop", None);
    stop_action.connect_activate(move |_, _| {
        stop_terminal(&terminal_clone2, &container_clone3);
        popover_clone3.popdown();
    });
    action_group.add_action(&stop_action);
    
    terminal.insert_action_group("split", Some(&action_group));
    terminal.insert_action_group("terminal", Some(&action_group));
    
    // Set up right-click gesture
    let gesture = gtk::GestureClick::new();
    gesture.set_button(3); // Right mouse button
    
    let popover_menu_clone = popover_menu.clone();
    gesture.connect_pressed(move |gesture, _n_press, x, y| {
        let _widget = gesture.widget();
        let rect = gtk::gdk::Rectangle::new(x as i32, y as i32, 1, 1);
        popover_menu_clone.set_pointing_to(Some(&rect));
        popover_menu_clone.popup();
    });
    
    terminal.add_controller(gesture);
}