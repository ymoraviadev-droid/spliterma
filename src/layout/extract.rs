use crate::layout::types::{SplitType, TerminalLayout};
use gtk4 as gtk;
use vte4::Terminal;
use vte4::prelude::*; // for TerminalExt::current_directory_uri()

// Make Paned first, then Box; Box falls through to child if it's just a wrapper.
pub fn extract_layout(widget: &gtk::Widget) -> Result<TerminalLayout, String> {
    // 1) Split containers
    if let Ok(paned) = widget.clone().downcast::<gtk::Paned>() {
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

    // 2) Terminal container OR transparent wrapper
    if let Ok(container) = widget.clone().downcast::<gtk::Box>() {
        if let Some(terminal) = find_terminal_in_container(&container) {
            // Prefer live cwd from VTE (updates after `cd`), fall back to stored data
            let live_cwd = terminal
                .current_directory_uri()
                .and_then(|u| file_uri_to_path(&u));

            let working_dir = if let Some(path) = live_cwd {
                path
            } else {
                unsafe {
                    terminal
                        .data::<String>("working_dir")
                        .map(|d| d.as_ref().clone())
                        .unwrap_or_else(|| {
                            std::env::current_dir()
                                .unwrap_or_default()
                                .to_string_lossy()
                                .to_string()
                        })
                }
            };

            let name = extract_terminal_name(&container).unwrap_or_else(|| "Terminal".to_string());
            let color_index = extract_color_index(&container);

            return Ok(TerminalLayout {
                name,
                color_index,
                working_dir,
                split_type: None,
                children: vec![],
            });
        }

        // Transparent wrapper: recurse into first child if any
        if let Some(child) = container.first_child() {
            return extract_layout(&child);
        }

        return Err("Empty gtk::Box with no children".to_string());
    }

    Err(format!("Unsupported widget type: {:?}", widget.type_()))
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

pub fn extract_color_index(container: &gtk::Box) -> usize {
    // `data::<usize>` returns Option<NonNull<usize>> in your setup,
    // so we must dereference it explicitly.
    unsafe {
        match container.data::<usize>("color_index") {
            Some(ptr) => *ptr.as_ref(),
            None => 0,
        }
    }
}

fn file_uri_to_path(uri: &str) -> Option<String> {
    if !uri.starts_with("file://") {
        return None;
    }
    let rest = &uri["file://".len()..];
    // strip host if present, keep path
    let path = if let Some(pos) = rest.find('/') {
        &rest[pos..]
    } else {
        rest
    };
    Some(path.to_string())
}
