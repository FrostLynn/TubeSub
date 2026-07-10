use gtk::prelude::*;
use gtk::glib;

pub fn setup_drop_target(widget: &impl IsA<gtk::Widget>, on_drop: impl Fn(&str) + 'static) {
    let drop_target = gtk::DropTarget::new(glib::types::Type::STRING, gtk::gdk::DragAction::COPY);

    drop_target.connect_drop(move |_, value, _, _| {
        if let Ok(path) = value.get::<String>() {
            if path.ends_with(".srt") {
                on_drop(&path);
                return true;
            }
        }
        false
    });

    widget.add_controller(drop_target);
}
