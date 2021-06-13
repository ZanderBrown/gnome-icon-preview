#[macro_use]
extern crate log;
#[macro_use]
extern crate gtk_macros;
#[macro_use]
extern crate serde_derive;

use gettextrs::*;
use gtk::{gio, glib};

mod application;
mod common;
mod config;
mod object_wrapper;
mod project;
mod settings;
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

    gtk::init().expect("Unable to start GTK 4");

    let res = gio::Resource::load(config::PKGDATADIR.to_owned() + "/resources.gresource").expect("Failed to initialize the resource file.");
    gio::resources_register(&res);

    Application::run();
}
