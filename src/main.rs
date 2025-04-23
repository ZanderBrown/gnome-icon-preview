use gettextrs::*;
use gtk::{gio, glib};

mod application;
mod common;
#[rustfmt::skip]
mod config;
mod project;
mod widgets;

use application::Application;
use config::{GETTEXT_PACKAGE, LOCALEDIR};

fn main() -> glib::ExitCode {
    pretty_env_logger::init();

    // Prepare i18n
    setlocale(LocaleCategory::LcAll, "");
    bindtextdomain(GETTEXT_PACKAGE, LOCALEDIR).expect("Could not bind text domain");
    textdomain(GETTEXT_PACKAGE).expect("Could not bind text domain");

    glib::set_application_name(&gettext("App Icon Preview"));

    gtk::init().expect("Unable to start GTK 4");

    let res = gio::Resource::load(config::PKGDATADIR.to_owned() + "/resources.gresource")
        .expect("Failed to initialize the resource file.");
    gio::resources_register(&res);

    Application::run()
}
