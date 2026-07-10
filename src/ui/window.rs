use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::{gio, glib};
use std::cell::RefCell;
use std::sync::Arc;

use crate::config::AppConfig;
use crate::ui::SettingsWindow;
use crate::youtube::api::{YouTubeClient, Video};
use crate::youtube::auth::OAuthManager;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct TubeSubWindow {
        pub stack: RefCell<Option<gtk::Stack>>,
        pub status_page: RefCell<Option<adw::StatusPage>>,
        pub video_list: RefCell<Option<gtk::ListBox>>,
        pub refresh_button: RefCell<Option<gtk::Button>>,
        pub client: RefCell<Option<Arc<YouTubeClient>>>,
        pub oauth: RefCell<Option<Arc<OAuthManager>>>,
        pub config: RefCell<Option<AppConfig>>,
        pub client_id_entry: RefCell<Option<gtk::Entry>>,
        pub client_secret_entry: RefCell<Option<gtk::Entry>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for TubeSubWindow {
        const NAME: &'static str = "TubeSubWindow";
        type Type = super::TubeSubWindow;
        type ParentType = adw::ApplicationWindow;
    }

    impl ObjectImpl for TubeSubWindow {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();
            obj.setup_ui();
        }
    }
    impl WidgetImpl for TubeSubWindow {}
    impl WindowImpl for TubeSubWindow {}
    impl ApplicationWindowImpl for TubeSubWindow {}
    impl AdwApplicationWindowImpl for TubeSubWindow {}
}

glib::wrapper! {
    pub struct TubeSubWindow(ObjectSubclass<imp::TubeSubWindow>)
        @extends gtk::Widget, gtk::Window, gtk::ApplicationWindow, adw::ApplicationWindow,
        @implements gio::ActionGroup, gio::ActionMap;
}

impl TubeSubWindow {
    pub fn new<P: glib::IsA<gtk::Application>>(application: &P) -> Self {
        glib::Object::builder()
            .property("application", application)
            .property("title", "TubeSub")
            .property("default-width", 800)
            .property("default-height", 600)
            .build()
    }

    fn setup_ui(&self) {
        let main_box = gtk::Box::new(gtk::Orientation::Vertical, 0);

        let header = adw::HeaderBar::new();
        header.set_title_widget(Some(&gtk::Label::new(Some("TubeSub"))));

        // Settings button
        let settings_button = gtk::Button::from_icon_name("preferences-system-symbolic");
        settings_button.set_tooltip_text(Some("Settings"));
        let window_ref = self.downgrade();
        settings_button.connect_clicked(move |_| {
            if let Some(window) = window_ref.upgrade() {
                let config = window.imp().config.borrow();
                if let Some(config) = config.as_ref() {
                    let settings = SettingsWindow::new(&window, config.clone());
                    settings.present();
                }
            }
        });
        header.pack_end(&settings_button);

        main_box.append(&header);

        let stack = gtk::Stack::new();
        stack.set_transition_type(gtk::StackTransitionType::Crossfade);

        let setup_page = self.create_setup_page();
        stack.add_named(&setup_page, Some("setup"));

        let auth_page = self.create_auth_page();
        stack.add_named(&auth_page, Some("auth"));

        let video_page = self.create_video_page();
        stack.add_named(&video_page, Some("videos"));

        main_box.append(&stack);

        adw::prelude::AdwApplicationWindowExt::set_content(self, Some(&main_box));

        *self.imp().stack.borrow_mut() = Some(stack);
        *self.imp().status_page.borrow_mut() = Some(auth_page);
        *self.imp().video_list.borrow_mut() = Some(video_page);

        // Initialize config
        match AppConfig::new() {
            Ok(config) => {
                *self.imp().config.borrow_mut() = Some(config);
                self.init_auth();
            }
            Err(e) => {
                eprintln!("Failed to initialize config: {}", e);
            }
        }
    }

    fn create_setup_page(&self) -> gtk::Box {
        let setup_box = gtk::Box::new(gtk::Orientation::Vertical, 24);
        setup_box.set_margin_top(48);
        setup_box.set_margin_bottom(48);
        setup_box.set_margin_start(48);
        setup_box.set_margin_end(48);
        setup_box.set_valign(gtk::Align::Center);
        setup_box.set_halign(gtk::Align::Center);

        let status = adw::StatusPage::new();
        status.set_title("Welcome to TubeSub");
        status.set_description(Some("Enter your YouTube API credentials to get started"));
        status.set_icon_name(Some("dialog-information-symbolic"));

        let content = gtk::Box::new(gtk::Orientation::Vertical, 16);
        content.set_halign(gtk::Align::Center);
        content.set_width_request(400);

        let group = adw::PreferencesGroup::new();
        group.set_title("YouTube API Credentials");
        group.set_description(Some("Get your credentials from Google Cloud Console -> APIs & Services -> Credentials"));

        // Client ID
        let client_id_row = adw::ActionRow::new();
        client_id_row.set_title("Client ID");
        let client_id_entry = gtk::Entry::new();
        client_id_entry.set_hexpand(true);
        client_id_entry.add_css_class("flat");
        client_id_row.add_suffix(&client_id_entry);
        group.add(&client_id_row);

        // Client Secret
        let client_secret_row = adw::ActionRow::new();
        client_secret_row.set_title("Client Secret");
        let client_secret_entry = gtk::Entry::new();
        client_secret_entry.set_hexpand(true);
        client_secret_entry.set_visibility(false);
        client_secret_entry.add_css_class("flat");
        client_secret_row.add_suffix(&client_secret_entry);
        group.add(&client_secret_row);

        content.append(&group);

        let save_btn = gtk::Button::with_label("Save & Continue");
        save_btn.add_css_class("suggested-action");
        save_btn.add_css_class("pill");
        save_btn.set_halign(gtk::Align::Center);
        save_btn.set_margin_top(12);

        let window_ref = self.downgrade();
        let id_entry = client_id_entry.clone();
        let secret_entry = client_secret_entry.clone();
        save_btn.connect_clicked(move |_| {
            if let Some(window) = window_ref.upgrade() {
                let client_id = id_entry.text().to_string();
                let client_secret = secret_entry.text().to_string();

                if client_id.is_empty() || client_secret.is_empty() {
                    let dialog = gtk::MessageDialog::new(
                        Some(&window),
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

                let creds = crate::config::Credentials {
                    client_id: Some(client_id),
                    client_secret: Some(client_secret),
                };

                let config = window.imp().config.borrow();
                if let Some(config) = config.as_ref() {
                    match config.save_credentials(&creds) {
                        Ok(_) => {
                            window.init_auth();
                        }
                        Err(e) => {
                            let dialog = gtk::MessageDialog::new(
                                Some(&window),
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
        });

        content.append(&save_btn);
        status.set_child(Some(&content));
        setup_box.append(&status);

        *self.imp().client_id_entry.borrow_mut() = Some(client_id_entry);
        *self.imp().client_secret_entry.borrow_mut() = Some(client_secret_entry);

        setup_box
    }

    fn create_auth_page(&self) -> adw::StatusPage {
        let status = adw::StatusPage::new();
        status.set_title("Welcome to TubeSub");
        status.set_description(Some("Sign in to manage your YouTube subtitles"));
        status.set_icon_name(Some("dialog-information-symbolic"));

        let sign_in_btn = gtk::Button::with_label("Sign In via Default Browser");
        sign_in_btn.add_css_class("suggested-action");
        sign_in_btn.add_css_class("pill");

        let window_ref = self.downgrade();
        sign_in_btn.connect_clicked(move |_| {
            if let Some(window) = window_ref.upgrade() {
                window.start_auth_flow();
            }
        });

        status.set_child(Some(&sign_in_btn));
        status
    }

    fn create_video_page(&self) -> gtk::ListBox {
        let video_box = gtk::Box::new(gtk::Orientation::Vertical, 12);
        video_box.set_margin_top(12);
        video_box.set_margin_bottom(12);
        video_box.set_margin_start(12);
        video_box.set_margin_end(12);

        let header_row = gtk::Box::new(gtk::Orientation::Horizontal, 12);
        let auth_label = gtk::Label::new(Some("Authenticated"));
        auth_label.add_css_class("dim-label");
        header_row.append(&auth_label);

        let refresh = gtk::Button::from_icon_name("view-refresh-symbolic");
        refresh.add_css_class("flat");
        let window_ref = self.downgrade();
        refresh.connect_clicked(move |_| {
            if let Some(window) = window_ref.upgrade() {
                window.refresh_videos();
            }
        });
        header_row.append(&refresh);
        video_box.append(&header_row);

        let list = gtk::ListBox::new();
        list.add_css_class("boxed-list");
        video_box.append(&list);

        *self.imp().refresh_button.borrow_mut() = Some(refresh);

        list
    }

    fn init_auth(&self) {
        let config = self.imp().config.borrow();
        if let Some(config) = config.as_ref() {
            if !config.has_valid_credentials() {
                // Show setup wizard
                if let Some(stack) = self.imp().stack.borrow().as_ref() {
                    stack.set_visible_child_name("setup");
                }
                return;
            }

            let creds = config.load_credentials();
            let client_id = creds.client_id.unwrap_or_default();
            let client_secret = creds.client_secret.unwrap_or_default();

            match OAuthManager::new(client_id, client_secret) {
                Ok(oauth) => {
                    if let Some(token) = oauth.load_token() {
                        let client = Arc::new(YouTubeClient::new(token.access_token));
                        *self.imp().client.borrow_mut() = Some(client);
                        *self.imp().oauth.borrow_mut() = Some(Arc::new(oauth));
                        self.show_video_list();
                    } else {
                        *self.imp().oauth.borrow_mut() = Some(Arc::new(oauth));
                    }
                }
                Err(e) => {
                    eprintln!("Failed to create OAuth manager: {}", e);
                }
            }
        }
    }

    fn start_auth_flow(&self) {
        let oauth = self.imp().oauth.borrow().clone();
        if let Some(oauth) = oauth {
            let window_ref = self.downgrade();
            let (sender, receiver) = glib::MainContext::channel::<Result<String, String>>(glib::Priority::DEFAULT);

            std::thread::spawn(move || {
                let result = oauth.start_auth_flow().map(|t| t.access_token).map_err(|e| e.to_string());
                sender.send(result).unwrap();
            });

            receiver.attach(None, move |result| {
                if let Some(window) = window_ref.upgrade() {
                    match result {
                        Ok(access_token) => {
                            let client = Arc::new(YouTubeClient::new(access_token));
                            *window.imp().client.borrow_mut() = Some(client);
                            window.show_video_list();
                        }
                        Err(e) => {
                            eprintln!("Auth failed: {}", e);
                        }
                    }
                }
                glib::ControlFlow::Break
            });
        }
    }

    fn show_video_list(&self) {
        // Switch to video page
        if let Some(stack) = self.imp().stack.borrow().as_ref() {
            stack.set_visible_child_name("videos");
        }
        self.refresh_videos();
    }

    fn refresh_videos(&self) {
        let client = self.imp().client.borrow().clone();
        if let Some(client) = client {
            let list = self.imp().video_list.borrow().clone();
            if let Some(list) = list {
                while let Some(child) = list.first_child() {
                    list.remove(&child);
                }

                let spinner = gtk::Spinner::new();
                spinner.start();
                list.append(&spinner);

                let window_ref = self.downgrade();
                let (sender, receiver) = glib::MainContext::channel::<Result<Vec<Video>, String>>(glib::Priority::DEFAULT);

                std::thread::spawn(move || {
                    let result = client.fetch_videos().map_err(|e| e.to_string());
                    sender.send(result).unwrap();
                });

                receiver.attach(None, move |result| {
                    if let Some(window) = window_ref.upgrade() {
                        match result {
                            Ok(videos) => {
                                window.populate_video_list(&videos);
                            }
                            Err(e) => {
                                eprintln!("Failed to fetch videos: {}", e);
                            }
                        }
                    }
                    glib::ControlFlow::Break
                });
            }
        }
    }

    fn populate_video_list(&self, videos: &[Video]) {
        if let Some(list) = self.imp().video_list.borrow().as_ref() {
            while let Some(child) = list.first_child() {
                list.remove(&child);
            }

            for video in videos {
                let row = self.create_video_row(video);
                list.append(&row);
            }

            if videos.is_empty() {
                let empty = adw::StatusPage::new();
                empty.set_title("No Videos Found");
                empty.set_description(Some("Upload videos to your YouTube channel to see them here"));
                empty.set_icon_name(Some("video-x-generic-symbolic"));
                list.append(&empty);
            }
        }
    }

    fn create_video_row(&self, video: &Video) -> adw::ActionRow {
        let row = adw::ActionRow::new();
        row.set_title(&video.title);
        row.set_subtitle(&format!("Video ID: {}", video.id));

        let button = gtk::Button::with_label("Add SRT");
        button.add_css_class("flat");

        let video_id = video.id.clone();
        let window_ref = self.downgrade();
        button.connect_clicked(move |_| {
            if let Some(window) = window_ref.upgrade() {
                window.select_srt_file(&video_id);
            }
        });

        row.add_suffix(&button);

        // Setup drag-and-drop for SRT files
        let video_id = video.id.clone();
        let window_ref = self.downgrade();
        let drop_target = gtk::DropTarget::new(glib::types::Type::STRING, gtk::gdk::DragAction::COPY);
        drop_target.connect_drop(move |_, value, _, _| {
            if let Ok(path) = value.get::<String>() {
                if path.ends_with(".srt") {
                    if let Some(window) = window_ref.upgrade() {
                        window.upload_srt(&video_id, std::path::Path::new(&path));
                    }
                    return true;
                }
            }
            false
        });
        row.add_controller(drop_target);

        row
    }

    fn select_srt_file(&self, video_id: &str) {
        let video_id = video_id.to_string();
        let window_ref = self.downgrade();

        let dialog = gtk::FileChooserDialog::new(
            Some("Select SRT File"),
            Some(self),
            gtk::FileChooserAction::Open,
            &[
                ("Cancel", gtk::ResponseType::Cancel),
                ("Open", gtk::ResponseType::Accept),
            ],
        );

        let filter = gtk::FileFilter::new();
        filter.add_pattern("*.srt");
        filter.set_name(Some("SRT Files"));
        dialog.add_filter(&filter);

        dialog.connect_response(move |dialog, response| {
            if response == gtk::ResponseType::Accept {
                if let Some(file) = dialog.file() {
                    if let Some(path) = file.path() {
                        if let Some(window) = window_ref.upgrade() {
                            window.upload_srt(&video_id, &path);
                        }
                    }
                }
            }
            dialog.close();
        });

        dialog.present();
    }

    fn upload_srt(&self, video_id: &str, path: &std::path::Path) {
        let client = self.imp().client.borrow().clone();
        if let Some(client) = client {
            let video_id = video_id.to_string();
            let path = path.to_path_buf();
            let window_ref = self.downgrade();
            let (sender, receiver) = glib::MainContext::channel::<Result<(), String>>(glib::Priority::DEFAULT);

            std::thread::spawn(move || {
                let result = (|| -> Result<(), String> {
                    let content = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
                    client.upload_caption(&video_id, &content, "en").map_err(|e| e.to_string())
                })();
                sender.send(result).unwrap();
            });

            receiver.attach(None, move |result| {
                if let Some(window) = window_ref.upgrade() {
                    match result {
                        Ok(_) => {
                            let dialog = gtk::MessageDialog::new(
                                Some(&window),
                                gtk::DialogFlags::MODAL,
                                gtk::MessageType::Info,
                                gtk::ButtonsType::Ok,
                                "Subtitle uploaded successfully!",
                            );
                            dialog.connect_response(|dialog, _| {
                                dialog.close();
                            });
                            dialog.present();
                        }
                        Err(e) => {
                            eprintln!("Upload failed: {}", e);
                        }
                    }
                }
                glib::ControlFlow::Break
            });
        }
    }
}
