mod app;
mod config;
mod subtitle;
mod ui;
mod youtube;

use app::TubeSubApplication;

fn main() {
    dotenv::dotenv().ok();
    tracing_subscriber::fmt::init();

    let app = TubeSubApplication::new();
    app.run();
}
