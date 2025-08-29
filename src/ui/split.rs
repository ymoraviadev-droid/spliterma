use crate::util::ids::next_terminal_number;
use crate::{constants::TERMINAL_COLORS, ui::terminal::create_terminal_with_title};
use gtk4 as gtk;
use vte4::Terminal; // <- מוסיף את טיפוס הטרמינל
use vte4::prelude::*; // <- traits ל- reset() וכו' // <- המונה הבטוח

pub fn split_terminal(current_container: &gtk::Box, orientation: gtk::Orientation) {
    let parent = current_container
        .parent()
        .expect("Container should have a parent");

    let paned = gtk::Paned::new(orientation);
    paned.set_hexpand(true);
    paned.set_vexpand(true);
    paned.set_wide_handle(true);

    // מונה טרמינלים בטוח (AtomicUsize) — בלי +1
    let terminal_num = next_terminal_number();
    let new_container = create_terminal_with_title(
        &format!("Terminal {}", terminal_num),
        terminal_num % TERMINAL_COLORS.len(),
        None,
    );

    if let Ok(window) = parent.clone().downcast::<gtk::ApplicationWindow>() {
        window.set_child(Some(&paned));
        paned.set_start_child(Some(current_container.upcast_ref::<gtk::Widget>()));
        paned.set_end_child(Some(new_container.upcast_ref::<gtk::Widget>()));
    } else if let Ok(parent_paned) = parent.downcast::<gtk::Paned>() {
        let current_widget = current_container.upcast_ref::<gtk::Widget>();
        let is_start_child = parent_paned.start_child().as_ref() == Some(current_widget);

        if is_start_child {
            parent_paned.set_start_child(gtk::Widget::NONE);
        } else {
            parent_paned.set_end_child(gtk::Widget::NONE);
        }

        paned.set_start_child(Some(current_container.upcast_ref::<gtk::Widget>()));
        paned.set_end_child(Some(new_container.upcast_ref::<gtk::Widget>()));

        if is_start_child {
            parent_paned.set_start_child(Some(paned.upcast_ref::<gtk::Widget>()));
        } else {
            parent_paned.set_end_child(Some(paned.upcast_ref::<gtk::Widget>()));
        }
    }

    // אופציונלי: קבע 50/50 אחרי realize
    paned.connect_realize(move |p| {
        match orientation {
            gtk::Orientation::Horizontal => {
                let w = p.allocated_width();
                if w > 0 {
                    p.set_position(w / 2);
                }
            }
            gtk::Orientation::Vertical => {
                let h = p.allocated_height();
                if h > 0 {
                    p.set_position(h / 2);
                }
            }
            _ => {} // חובה: enum לא ממצה
        }
    });
}

pub fn stop_terminal(terminal: &Terminal, container: &gtk::Box) {
    terminal.reset(true, true);

    if let Some(parent) = container.parent() {
        if let Ok(parent_paned) = parent.clone().downcast::<gtk::Paned>() {
            let is_start_child =
                parent_paned.start_child().as_ref() == Some(container.upcast_ref::<gtk::Widget>());

            if is_start_child {
                if let Some(other_child) = parent_paned.end_child() {
                    replace_paned_with_child(&parent_paned, &other_child);
                }
            } else {
                if let Some(other_child) = parent_paned.start_child() {
                    replace_paned_with_child(&parent_paned, &other_child);
                }
            }
        } else if let Ok(window) = parent.downcast::<gtk::ApplicationWindow>() {
            window.close();
        }
    }
}

fn replace_paned_with_child(paned: &gtk::Paned, remaining_child: &gtk::Widget) {
    if let Some(grandparent) = paned.parent() {
        if let Ok(grandparent_paned) = grandparent.clone().downcast::<gtk::Paned>() {
            if paned.start_child().as_ref() == Some(remaining_child) {
                paned.set_start_child(gtk::Widget::NONE);
            } else {
                paned.set_end_child(gtk::Widget::NONE);
            }

            let is_start_child =
                grandparent_paned.start_child().as_ref() == Some(paned.upcast_ref::<gtk::Widget>());

            if is_start_child {
                grandparent_paned.set_start_child(gtk::Widget::NONE);
                grandparent_paned.set_start_child(Some(remaining_child));
            } else {
                grandparent_paned.set_end_child(gtk::Widget::NONE);
                grandparent_paned.set_end_child(Some(remaining_child));
            }
        } else if let Ok(window) = grandparent.downcast::<gtk::ApplicationWindow>() {
            paned.set_start_child(gtk::Widget::NONE);
            paned.set_end_child(gtk::Widget::NONE);
            window.set_child(Some(remaining_child));
        }
    }
}
