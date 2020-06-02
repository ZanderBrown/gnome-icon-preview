#[macro_use]
extern crate log;
#[macro_use]
extern crate glib;
#[macro_use]
extern crate gtk_macros;
#[macro_use]
extern crate strum_macros;
#[macro_use]
extern crate serde_derive;

use gettextrs::*;

mod application;
mod common;
mod config;
mod object_wrapper;
mod project;
mod settings;
mod static_resources;
mod widgets;

use application::Application;
use config::{GETTEXT_PACKAGE, LOCALEDIR};

fn main() {
    pretty_env_logger::init();

    // Prepare i18n
    setlocale(LocaleCategory::LcAll, "");
    bindtextdomain(GETTEXT_PACKAGE, LOCALEDIR);
    textdomain(GETTEXT_PACKAGE);

    glib::set_application_name(&format!("App Icon Preview{}", config::NAME_SUFFIX));
    glib::set_prgname(Some("app-icon-preview"));

    gtk::init().expect("Unable to start GTK3");
    libhandy::Column::new();

    static_resources::init().expect("Failed to initialize the resource file.");

    let app = Application::new();
    app.run(app.clone());
}
