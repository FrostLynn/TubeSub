use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::{gio, glib};

use crate::config::{AppConfig, Credentials};

mod imp {
    use super::*;
    use std::cell::RefCell;

    #[derive(Debug, Default)]
    pub struct SettingsWindow {
        pub client_id_entry: RefCell<Option<gtk::Entry>>,
        pub client_secret_entry: RefCell<Option<gtk::Entry>>,
        pub config: RefCell<Option<AppConfig>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SettingsWindow {
        const NAME: &'static str = "SettingsWindow";
        type Type = super::SettingsWindow;
        type ParentType = adw::PreferencesWindow;
    }

    impl ObjectImpl for SettingsWindow {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();
            obj.setup_ui();
        }
    }
    impl WidgetImpl for SettingsWindow {}
    impl WindowImpl for SettingsWindow {}
    impl AdwWindowImpl for SettingsWindow {}
    impl PreferencesWindowImpl for SettingsWindow {}
}

glib::wrapper! {
    pub struct SettingsWindow(ObjectSubclass<imp::SettingsWindow>)
        @extends gtk::Widget, gtk::Window, adw::Window, adw::PreferencesWindow,
        @implements gio::ActionGroup, gio::ActionMap;
}

impl SettingsWindow {
    pub fn new<P: glib::IsA<gtk::Window>>(parent: &P, config: AppConfig) -> Self {
        let window: Self = glib::Object::builder()
            .property("transient-for", parent)
            .property("title", "Settings")
            .property("modal", true)
            .property("search-enabled", false)
            .build();

        *window.imp().config.borrow_mut() = Some(config);
        window.load_credentials();
        window
    }

    fn setup_ui(&self) {
        let page = adw::PreferencesPage::new();
        page.set_title("General");
        page.set_icon_name(Some("preferences-system-symbolic"));
        self.add(&page);

        // YouTube API Credentials group
        let creds_group = adw::PreferencesGroup::new();
        creds_group.set_title("YouTube API Credentials");
        creds_group.set_description(Some("Enter your Google Cloud Console credentials. Get them from https://console.cloud.google.com/apis/credentials"));
        page.add(&creds_group);

        // Client ID row
        let client_id_row = adw::ActionRow::new();
        client_id_row.set_title("Client ID");
        let client_id_entry = gtk::Entry::new();
        client_id_entry.set_hexpand(true);
        client_id_entry.add_css_class("flat");
        client_id_row.add_suffix(&client_id_entry);
        creds_group.add(&client_id_row);

        // Client Secret row
        let client_secret_row = adw::ActionRow::new();
        client_secret_row.set_title("Client Secret");
        let client_secret_entry = gtk::Entry::new();
        client_secret_entry.set_hexpand(true);
        client_secret_entry.set_visibility(false);
        client_secret_entry.add_css_class("flat");
        client_secret_row.add_suffix(&client_secret_entry);
        creds_group.add(&client_secret_row);

        *self.imp().client_id_entry.borrow_mut() = Some(client_id_entry);
        *self.imp().client_secret_entry.borrow_mut() = Some(client_secret_entry);

        // Save button
        let save_group = adw::PreferencesGroup::new();
        page.add(&save_group);

        let save_button = gtk::Button::with_label("Save");
        save_button.add_css_class("suggested-action");
        save_button.set_margin_top(12);
        save_button.set_halign(gtk::Align::End);

        let window_ref = self.downgrade();
        save_button.connect_clicked(move |_| {
            if let Some(window) = window_ref.upgrade() {
                window.save_credentials();
            }
        });

        save_group.add(&save_button);
    }

    fn load_credentials(&self) {
        let config = self.imp().config.borrow();
        if let Some(config) = config.as_ref() {
            let creds = config.load_credentials();

            if let Some(entry) = self.imp().client_id_entry.borrow().as_ref() {
                if let Some(cid) = &creds.client_id {
                    entry.set_text(cid);
                }
            }

            if let Some(entry) = self.imp().client_secret_entry.borrow().as_ref() {
                if let Some(csecret) = &creds.client_secret {
                    entry.set_text(csecret);
                }
            }
        }
    }

    fn save_credentials(&self) {
        let client_id = self
            .imp()
            .client_id_entry
            .borrow()
            .as_ref()
            .map(|e| e.text().to_string())
            .unwrap_or_default();

        let client_secret = self
            .imp()
            .client_secret_entry
            .borrow()
            .as_ref()
            .map(|e| e.text().to_string())
            .unwrap_or_default();

        if client_id.is_empty() || client_secret.is_empty() {
            let dialog = gtk::MessageDialog::new(
                Some(self),
                gtk::DialogFlags::MODAL,
                gtk::MessageType::Warning,
                gtk::ButtonsType::Ok,
                "Please enter both Client ID and Client Secret.",
            );
            dialog.connect_response(|dialog, _| {
                dialog.close();
            });
            dialog.present();
            return;
        }

        let creds = Credentials {
            client_id: Some(client_id),
            client_secret: Some(client_secret),
        };

        let config = self.imp().config.borrow();
        if let Some(config) = config.as_ref() {
            match config.save_credentials(&creds) {
                Ok(_) => {
                    self.close();
                }
                Err(e) => {
                    let dialog = gtk::MessageDialog::new(
                        Some(self),
                        gtk::DialogFlags::MODAL,
                        gtk::MessageType::Error,
                        gtk::ButtonsType::Ok,
                        &format!("Failed to save credentials: {}", e),
                    );
                    dialog.connect_response(|dialog, _| {
                        dialog.close();
                    });
                    dialog.present();
                }
            }
        }
    }
}
