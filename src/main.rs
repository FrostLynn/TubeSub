mod app;
mod config;
mod subtitle;
mod ui;
mod youtube;

use app::TubeSubApplication;

#[cfg(target_os = "windows")]
fn set_gsettings_schema_dir() {
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let schema_dir = exe_dir.join("share").join("glib-2.0").join("schemas");
            if schema_dir.exists() {
                std::env::set_var("GSETTINGS_SCHEMA_DIR", &schema_dir);
            }
        }
    }
}

fn main() {
    dotenv::dotenv().ok();
    tracing_subscriber::fmt::init();

    #[cfg(target_os = "windows")]
    set_gsettings_schema_dir();

    let app = TubeSubApplication::new();
    app.run();
}
