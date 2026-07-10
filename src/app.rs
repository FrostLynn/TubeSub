use adw::prelude::*;
use gtk::gio;

use crate::ui::TubeSubWindow;

pub struct TubeSubApplication {
    app: adw::Application,
}

impl TubeSubApplication {
    pub fn new() -> Self {
        let app = adw::Application::new(
            Some("com.tubesub.app"),
            gio::ApplicationFlags::FLAGS_NONE,
        );

        let application = Self { app };

        application.app.connect_activate(|app| {
            let window = TubeSubWindow::new(app);
            gtk::prelude::GtkWindowExt::present(&window);
        });

        application
    }

    pub fn run(&self) {
        self.app.run();
    }
}
