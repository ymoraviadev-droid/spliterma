fn count_terminals() -> usize {
    // Simple counter for naming - in a real app you might want to track this better
    static mut COUNTER: usize = 1;
    unsafe {
        COUNTER += 1;
        COUNTER
    }
}

fn setup_global_menu(window: &gtk::ApplicationWindow, _root_container: &gtk::Box) {
    let menubar = gio::Menu::new();
    let file_menu = gio::Menu::new();

    file_menu.append(Some("Save Layout"), Some("app.save-layout"));
    file_menu.append(Some("Load Layout"), Some("app.load-layout"));
    menubar.append_submenu(Some("File"), &file_menu);

    let app = window.application().unwrap();

    // Save: always use the current window child (not the initial container)
    let save_action = gio::SimpleAction::new("save-layout", None);
    let win_for_save = window.clone();
    save_action.connect_activate(move |_, _| {
        if let Some(child) = win_for_save.child() {
            save_layout(&child);
        }
    });
    app.add_action(&save_action);

    // Load (you already had this)
    let load_action = gio::SimpleAction::new("load-layout", None);
    let win_for_load = window.clone();
    load_action.connect_activate(move |_, _| {
        load_layout(&win_for_load);
    });
    app.add_action(&load_action);

    // Shortcuts: Ctrl+S / Ctrl+O
    app.set_accels_for_action("app.save-layout", &["<Primary>S"]);
    app.set_accels_for_action("app.load-layout", &["<Primary>O"]);

    app.set_menubar(Some(&menubar));
}

use gtk::{gio, glib};
use gtk4 as gtk;
use gtk::prelude::*;
use vte4::prelude::*;
use vte4::{PtyFlags, Terminal};
use serde::{Deserialize, Serialize};
use std::fs;

static TERMINAL_COLORS: &[&str] = &[
    "#3584E4", // Blue
    "#33D17A", // Green  
    "#F6D32D", // Yellow
    "#FF7800", // Orange
    "#E01B24", // Red
    "#9141AC", // Purple
    "#986A44", // Brown
    "#5E5C64", // Gray
];

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TerminalLayout {
    name: String,
    color_index: usize,
    working_dir: String,
    split_type: Option<SplitType>,
    children: Vec<TerminalLayout>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum SplitType {
    Horizontal,
    Vertical,
}

#[derive(Debug, Serialize, Deserialize)]
struct SavedLayout {
    version: String,
    root: TerminalLayout,
}

fn main() {
    let app = gtk::Application::builder()
        .application_id("com.spliterma.app")
        .build();

    app.connect_activate(|app| {
        // Window
        let window = gtk::ApplicationWindow::builder()
            .application(app)
            .title("Spliterma")
            .default_width(1000)
            .default_height(700)
            .build();

        // Create initial terminal with title bar
        let terminal_container = create_terminal_with_title("Terminal 1", 0, None);
        
        // Set up global menu bar
        setup_global_menu(&window, &terminal_container);
        
        // Set terminal container directly as window child
        window.set_child(Some(&terminal_container));
        window.present();
    });

    app.run();
}

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

fn create_terminal_with_title(title: &str, color_index: usize, working_dir: Option<&str>) -> gtk::Box {
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

fn stop_terminal(terminal: &Terminal, container: &gtk::Box) {
    // Try to kill the terminal process - this is a simplified approach
    // In VTE4, we don't have direct access to child_process_id, so we'll just reset the terminal
    terminal.reset(true, true);
    
    // Find and remove this terminal container from its parent
    if let Some(parent) = container.parent() {
        if let Ok(parent_paned) = parent.clone().downcast::<gtk::Paned>() {
            // This container is in a paned widget
            let is_start_child = parent_paned.start_child().as_ref() == Some(container.upcast_ref::<gtk::Widget>());
            
            if is_start_child {
                // Move the end child to replace the entire paned
                if let Some(other_child) = parent_paned.end_child() {
                    replace_paned_with_child(&parent_paned, &other_child);
                }
            } else {
                // Move the start child to replace the entire paned
                if let Some(other_child) = parent_paned.start_child() {
                    replace_paned_with_child(&parent_paned, &other_child);
                }
            }
        } else if let Ok(window) = parent.downcast::<gtk::ApplicationWindow>() {
            // This is the last terminal in the window - could close app or show message
            window.close();
        }
    }
}

fn replace_paned_with_child(paned: &gtk::Paned, remaining_child: &gtk::Widget) {
    if let Some(grandparent) = paned.parent() {
        if let Ok(grandparent_paned) = grandparent.clone().downcast::<gtk::Paned>() {
            // Remove remaining child from current paned
            if paned.start_child().as_ref() == Some(remaining_child) {
                paned.set_start_child(gtk::Widget::NONE);
            } else {
                paned.set_end_child(gtk::Widget::NONE);
            }
            
            // Replace the paned with the remaining child in grandparent
            let is_start_child = grandparent_paned.start_child().as_ref() == Some(paned.upcast_ref::<gtk::Widget>());
            
            if is_start_child {
                grandparent_paned.set_start_child(gtk::Widget::NONE);
                grandparent_paned.set_start_child(Some(remaining_child));
            } else {
                grandparent_paned.set_end_child(gtk::Widget::NONE);
                grandparent_paned.set_end_child(Some(remaining_child));
            }
        } else if let Ok(window) = grandparent.downcast::<gtk::ApplicationWindow>() {
            // Replace entire window content with remaining child
            paned.set_start_child(gtk::Widget::NONE);
            paned.set_end_child(gtk::Widget::NONE);
            window.set_child(Some(remaining_child));
        }
    }
}

fn split_terminal(current_container: &gtk::Box, orientation: gtk::Orientation) {
    // Get the current parent - could be window, paned, or other container
    let parent = current_container.parent().expect("Container should have a parent");
    
    // Create new paned widget with the desired orientation
    let paned = gtk::Paned::new(orientation);
    paned.set_hexpand(true);
    paned.set_vexpand(true);
    
    // Make the paned border thicker and more visible
    paned.set_wide_handle(true);
    
    // Create new terminal with title bar
    let terminal_count = count_terminals() + 1;
    let new_container = create_terminal_with_title(&format!("Terminal {}", terminal_count), terminal_count % TERMINAL_COLORS.len(), None);
    
    // Handle different parent types
    if let Ok(window) = parent.clone().downcast::<gtk::ApplicationWindow>() {
        // Parent is the main window
        window.set_child(Some(&paned));
        paned.set_start_child(Some(current_container.upcast_ref::<gtk::Widget>()));
        paned.set_end_child(Some(new_container.upcast_ref::<gtk::Widget>()));
        
    } else if let Ok(parent_paned) = parent.downcast::<gtk::Paned>() {
        // Parent is another paned widget - we need to replace current container
        
        // Determine which child of the parent paned we are
        let current_widget = current_container.upcast_ref::<gtk::Widget>();
        let is_start_child = parent_paned.start_child().as_ref() == Some(current_widget);
        
        // Remove current container from parent paned
        if is_start_child {
            parent_paned.set_start_child(gtk::Widget::NONE);
        } else {
            parent_paned.set_end_child(gtk::Widget::NONE);
        }
        
        // Set up new paned with current container and new container
        paned.set_start_child(Some(current_container.upcast_ref::<gtk::Widget>()));
        paned.set_end_child(Some(new_container.upcast_ref::<gtk::Widget>()));
        
        // Add new paned to parent in place of current container
        if is_start_child {
            parent_paned.set_start_child(Some(paned.upcast_ref::<gtk::Widget>()));
        } else {
            parent_paned.set_end_child(Some(paned.upcast_ref::<gtk::Widget>()));
        }
    }
    
    // Set equal sizing for the split (50/50) - do this after adding to parent
    paned.connect_realize(move |paned| {
        match orientation {
            gtk::Orientation::Horizontal => {
                let width = paned.allocated_width();
                if width > 0 {
                    paned.set_position(width / 2);
                }
            },
            gtk::Orientation::Vertical => {
                let height = paned.allocated_height();
                if height > 0 {
                    paned.set_position(height / 2);
                }
            },
            _ => {}
        }
    });
}

fn save_layout(root_widget: &gtk::Widget) {
    let dialog = gtk::FileChooserDialog::builder()
        .title("Save Layout")
        .action(gtk::FileChooserAction::Save)
        .modal(true)
        .build();

    dialog.add_button("Cancel", gtk::ResponseType::Cancel);
    dialog.add_button("Save", gtk::ResponseType::Accept);

    // החלף root_container ב-root_widget
    if let Some(window) = root_widget
        .root()
        .and_then(|root| root.downcast::<gtk::Window>().ok())
    {
        dialog.set_transient_for(Some(&window));
    }

    // נשמור עותק חזק לשימוש בתוך ה-closure
    let root_widget_clone = root_widget.clone();
    dialog.connect_response(move |dialog, response| {
        if response == gtk::ResponseType::Accept {
            if let Some(file) = dialog.file() {
                if let Some(path) = file.path() {
                    match extract_layout(&root_widget_clone) {
                        Ok(layout) => {
                            let saved_layout = SavedLayout {
                                version: "1.0".to_string(),
                                root: layout,
                            };

                            match serde_json::to_string_pretty(&saved_layout) {
                                Ok(json) => {
                                    if let Err(e) = std::fs::write(&path, json) {
                                        eprintln!("Failed to save layout: {}", e);
                                        show_error_dialog("Failed to save layout", &e.to_string());
                                    } else {
                                        println!("Layout saved to: {}", path.display());
                                    }
                                }
                                Err(e) => {
                                    eprintln!("Failed to serialize layout: {}", e);
                                    show_error_dialog("Failed to serialize layout", &e.to_string());
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("Failed to extract layout: {}", e);
                            show_error_dialog("Failed to extract layout", &e);
                        }
                    }
                }
            }
        }
        dialog.close();
    });

    dialog.present();
}


fn load_layout(window: &gtk::ApplicationWindow) {
    let dialog = gtk::FileChooserDialog::builder()
        .title("Load Layout")
        .action(gtk::FileChooserAction::Open)
        .modal(true)
        .build();
        
    dialog.add_button("Cancel", gtk::ResponseType::Cancel);
    dialog.add_button("Load", gtk::ResponseType::Accept);
    dialog.set_transient_for(Some(window));
    
    // Add JSON filter
    let filter = gtk::FileFilter::new();
    filter.add_pattern("*.json");
    filter.set_name(Some("JSON Layout Files"));
    dialog.add_filter(&filter);
    
    let window_clone = window.clone();
    dialog.connect_response(move |dialog, response| {
        if response == gtk::ResponseType::Accept {
            if let Some(file) = dialog.file() {
                if let Some(path) = file.path() {
                    match fs::read_to_string(&path) {
                        Ok(json) => {
                            match serde_json::from_str::<SavedLayout>(&json) {
                                Ok(saved_layout) => {
                                    match build_layout_from_data(&saved_layout.root) {
                                        Ok(new_container) => {
                                            window_clone.set_child(Some(&new_container));
                                            println!("Layout loaded from: {}", path.display());
                                        }
                                        Err(e) => {
                                            eprintln!("Failed to build layout: {}", e);
                                            show_error_dialog("Failed to build layout", &e);
                                        }
                                    }
                                }
                                Err(e) => {
                                    eprintln!("Failed to parse layout file: {}", e);
                                    show_error_dialog("Failed to parse layout file", &e.to_string());
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("Failed to read layout file: {}", e);
                            show_error_dialog("Failed to read file", &e.to_string());
                        }
                    }
                }
            }
        }
        dialog.close();
    });
    
    dialog.present();
}

fn extract_layout(widget: &gtk::Widget) -> Result<TerminalLayout, String> {
    if let Some(container) = widget.downcast_ref::<gtk::Box>() {
        // This is a terminal container
        if let Some(terminal) = find_terminal_in_container(container) {
            let working_dir = unsafe {
                terminal.data::<String>("working_dir")
                    .and_then(|data| Some(data.as_ref().clone()))
                    .unwrap_or_else(|| std::env::current_dir().unwrap_or_default().to_string_lossy().to_string())
            };
                
            // Get terminal name from title bar
            let name = extract_terminal_name(container).unwrap_or_else(|| "Terminal".to_string());
            let color_index = extract_color_index(container);
            
            return Ok(TerminalLayout {
                name,
                color_index,
                working_dir,
                split_type: None,
                children: vec![],
            });
        }
    } else if let Some(paned) = widget.downcast_ref::<gtk::Paned>() {
        // This is a split container
        let split_type = match paned.orientation() {
            gtk::Orientation::Horizontal => SplitType::Horizontal,
            gtk::Orientation::Vertical => SplitType::Vertical,
            _ => return Err("Unknown orientation".to_string()),
        };
        
        let mut children = Vec::new();
        
        if let Some(start_child) = paned.start_child() {
            children.push(extract_layout(&start_child)?);
        }
        
        if let Some(end_child) = paned.end_child() {
            children.push(extract_layout(&end_child)?);
        }
        
        return Ok(TerminalLayout {
            name: "Split".to_string(),
            color_index: 0,
            working_dir: String::new(),
            split_type: Some(split_type),
            children,
        });
    }
    
    Err("Unknown widget type".to_string())
}

fn build_layout_from_data(layout: &TerminalLayout) -> Result<gtk::Box, String> {
    if layout.split_type.is_none() {
        // This is a terminal
        let container = create_terminal_with_title(
            &layout.name,
            layout.color_index,
            Some(&layout.working_dir)
        );
        Ok(container)
    } else {
        // This is a split - we need to create a paned with children
        if layout.children.len() != 2 {
            return Err("Split must have exactly 2 children".to_string());
        }
        
        let orientation = match layout.split_type {
            Some(SplitType::Horizontal) => gtk::Orientation::Horizontal,
            Some(SplitType::Vertical) => gtk::Orientation::Vertical,
            None => return Err("No split type specified".to_string()),
        };
        
        let paned = gtk::Paned::new(orientation);
        paned.set_hexpand(true);
        paned.set_vexpand(true);
        paned.set_wide_handle(true);
        
        let start_child = build_layout_from_data(&layout.children[0])?;
        let end_child = build_layout_from_data(&layout.children[1])?;
        
        paned.set_start_child(Some(start_child.upcast_ref::<gtk::Widget>()));
        paned.set_end_child(Some(end_child.upcast_ref::<gtk::Widget>()));
        
        // Wrap paned in a box to match our container structure
        let wrapper = gtk::Box::new(gtk::Orientation::Vertical, 0);
        wrapper.set_hexpand(true);
        wrapper.set_vexpand(true);
        wrapper.append(&paned);
        
        Ok(wrapper)
    }
}

fn find_terminal_in_container(container: &gtk::Box) -> Option<Terminal> {
    // Look through container children to find the terminal
    let mut child = container.first_child();
    while let Some(widget) = child {
        if let Some(terminal) = widget.clone().downcast::<Terminal>().ok() {
            return Some(terminal);
        }
        child = widget.next_sibling();
    }
    None
}

fn extract_terminal_name(container: &gtk::Box) -> Option<String> {
    // Find the title label in the container
    let mut child = container.first_child();
    while let Some(widget) = child {
        if let Some(title_bar) = widget.clone().downcast::<gtk::Box>().ok() {
            let mut title_child = title_bar.first_child();
            while let Some(title_widget) = title_child {
                if let Some(label) = title_widget.clone().downcast::<gtk::Label>().ok() {
                    return Some(label.text().to_string());
                }
                title_child = title_widget.next_sibling();
            }
        }
        child = widget.next_sibling();
    }
    None
}

fn extract_color_index(_container: &gtk::Box) -> usize {
    // For now, return a default color index
    // In a full implementation, you might want to store this as widget data
    0
}

fn show_error_dialog(title: &str, message: &str) {
    let dialog = gtk::MessageDialog::builder()
        .message_type(gtk::MessageType::Error)
        .title(title)
        .text(message)
        .modal(true)
        .build();
        
    dialog.add_button("OK", gtk::ResponseType::Ok);
    dialog.connect_response(|dialog, _| {
        dialog.close();
    });
    dialog.present();
}