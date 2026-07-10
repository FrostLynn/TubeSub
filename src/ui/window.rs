use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::{gio, glib};
use std::cell::RefCell;
use std::sync::Arc;

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
        main_box.append(&header);

        let stack = gtk::Stack::new();
        stack.set_transition_type(gtk::StackTransitionType::Crossfade);

        let auth_page = self.create_auth_page();
        stack.add_named(&auth_page, Some("auth"));

        let video_page = self.create_video_page();
        stack.add_named(&video_page, Some("videos"));

        main_box.append(&stack);

        adw::prelude::AdwApplicationWindowExt::set_content(self, Some(&main_box));

        *self.imp().stack.borrow_mut() = Some(stack);
        *self.imp().status_page.borrow_mut() = Some(auth_page);
        *self.imp().video_list.borrow_mut() = Some(video_page);

        self.init_auth();
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
        let oauth = OAuthManager::new().expect("Failed to create OAuth manager");

        if let Some(token) = oauth.load_token() {
            let client = Arc::new(YouTubeClient::new(token.access_token));
            *self.imp().client.borrow_mut() = Some(client);
            *self.imp().oauth.borrow_mut() = Some(Arc::new(oauth));
            self.show_video_list();
        } else {
            *self.imp().oauth.borrow_mut() = Some(Arc::new(oauth));
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
