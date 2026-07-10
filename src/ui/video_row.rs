use adw::prelude::*;

pub struct VideoRow {
    pub widget: adw::ActionRow,
}

impl VideoRow {
    pub fn new(title: &str) -> Self {
        let row = adw::ActionRow::new();
        row.set_title(title);
        row.set_subtitle("Click to add subtitle");

        let button = gtk::Button::with_label("Add SRT");
        button.add_css_class("flat");
        row.add_suffix(&button);

        Self { widget: row }
    }
}
