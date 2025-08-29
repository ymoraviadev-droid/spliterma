use crate::layout::types::{SplitType, TerminalLayout};
use gtk::prelude::*;
use gtk4 as gtk;
use vte4::Terminal;

pub(crate) fn extract_layout(widget: &gtk::Widget) -> Result<TerminalLayout, String> {
    if let Some(container) = widget.downcast_ref::<gtk::Box>() {
        // This is a terminal container
        if let Some(terminal) = find_terminal_in_container(container) {
            let working_dir = unsafe {
                terminal
                    .data::<String>("working_dir")
                    .and_then(|data| Some(data.as_ref().clone()))
                    .unwrap_or_else(|| {
                        std::env::current_dir()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .to_string()
                    })
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
