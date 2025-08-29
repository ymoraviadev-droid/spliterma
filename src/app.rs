use gtk::prelude::*;
use gtk4 as gtk;

use crate::ui::{menus::setup_global_menu, terminal::create_terminal_with_title};

pub fn run() {
    let app = gtk::Application::builder()
        .application_id("com.spliterma.app")
        .build();

    app.connect_activate(|app| {
        let window = gtk::ApplicationWindow::builder()
            .application(app)
            .title("Spliterma")
            .default_width(1000)
            .default_height(700)
            .build();

        // טרמינל פתיחה
        let initial = create_terminal_with_title("Terminal 1", 0, None);

        // תפריט עליון + קיצורי מקלדת
        setup_global_menu(&window);

        window.set_child(Some(&initial));
        window.present();
    });

    app.run();
}
