use gtk::prelude::*;
use gtk4 as gtk;

pub fn show_error_dialog(title: &str, message: &str) {
    let dialog = gtk::MessageDialog::builder()
        .message_type(gtk::MessageType::Error)
        .title(title)
        .text(message)
        .modal(true)
        .build();

    dialog.add_button("OK", gtk::ResponseType::Ok);
    dialog.connect_response(|d, _| d.close());
    dialog.present();
}
