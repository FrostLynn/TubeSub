use adw::prelude::*;

pub struct AuthStatusPage {
    pub widget: adw::StatusPage,
}

impl AuthStatusPage {
    pub fn new() -> Self {
        let status = adw::StatusPage::new();
        status.set_title("Welcome to TubeSub");
        status.set_description(Some("Sign in to manage your YouTube subtitles"));
        status.set_icon_name(Some("dialog-information-symbolic"));

        let sign_in_btn = gtk::Button::with_label("Sign In via Default Browser");
        sign_in_btn.add_css_class("suggested-action");
        sign_in_btn.add_css_class("pill");
        status.set_child(Some(&sign_in_btn));

        Self { widget: status }
    }
}
