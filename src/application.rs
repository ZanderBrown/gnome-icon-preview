use adw::{prelude::*, subclass::prelude::*};
use gettextrs::gettext;
use gtk::{gdk, gio, glib};

use crate::{config, project::Project, widgets::Window};

mod imp {
    use std::cell::OnceCell;

    use super::*;

    #[derive(Default)]
    pub struct Application {
        pub icon_theme: OnceCell<gtk::IconTheme>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Application {
        const NAME: &'static str = "Application";
        type ParentType = adw::Application;
        type Type = super::Application;
    }
    impl ObjectImpl for Application {}
    impl ApplicationImpl for Application {
        fn startup(&self) {
            self.parent_startup();
            // setup icon theme cache
            if let Some(display) = gdk::Display::default() {
                let icon_theme = gtk::IconTheme::for_display(&display);
                if let Err(err) = crate::common::init_tmp(&icon_theme) {
                    log::error!("Failed to load icon theme: {}", err);
                };
                self.icon_theme.set(icon_theme).unwrap();
            }
            let app = self.obj();
            let new_window = gio::ActionEntry::builder("new-window")
                .activate(move |app: &Self::Type, _, _| {
                    let window = app.create_window();
                    window.present();
                })
                .build();
            let quit = gio::ActionEntry::builder("quit")
                .activate(move |app: &Self::Type, _, _| app.quit())
                .build();

            let about = gio::ActionEntry::builder("about")
                .activate(move |app: &Self::Type, _, _| app.show_about_dialog())
                .build();

            app.add_action_entries([new_window, quit, about]);

            // Accelerators
            app.set_accels_for_action("win.open", &["<Control>o"]);
            app.set_accels_for_action("win.refresh", &["<Control>r"]);
            app.set_accels_for_action("win.shuffle", &["<Control>x"]);
            app.set_accels_for_action("win.export", &["<Control>e"]);
            app.set_accels_for_action("win.save-screenshot", &["<Control>s"]);
            app.set_accels_for_action("win.copy-screenshot", &["<Control>c"]);
            app.set_accels_for_action("app.quit", &["<Control>q"]);
            app.set_accels_for_action("app.new-window", &["<Control>n"]);
        }

        fn activate(&self) {
            self.parent_activate();
            let application = self.obj();
            let window = application.create_window();
            window.present();
        }

        fn open(&self, files: &[gio::File], _hint: &str) {
            let application = self.obj();
            for file in files.iter() {
                if let Ok(project) = Project::parse(file.clone(), true) {
                    let window = application.create_window();
                    window.set_open_project(project);
                    window.present();
                }
            }
        }
    }

    impl GtkApplicationImpl for Application {}
    impl AdwApplicationImpl for Application {}
}

glib::wrapper! {
    pub struct Application(ObjectSubclass<imp::Application>)
        @extends gio::Application, gtk::Application,
        @implements gio::ActionMap, gio::ActionGroup;
}

impl Application {
    pub fn run() {
        log::info!("App Icon Preview ({})", config::APP_ID);
        log::info!("Version: {} ({})", config::VERSION, config::PROFILE);
        log::info!("Datadir: {}", config::PKGDATADIR);

        let app = glib::Object::builder::<Self>()
            .property("application-id", config::APP_ID)
            .property("flags", gio::ApplicationFlags::HANDLES_OPEN)
            .property("resource-base-path", "/org/gnome/design/AppIconPreview")
            .build();
        app.run();
    }

    fn create_window(&self) -> Window {
        let group = gtk::WindowGroup::new();
        let window = Window::new(self);

        group.add_window(&window);

        window
    }

    fn show_about_dialog(&self) {
        let window = self.active_window().and_downcast::<Window>().unwrap();
        adw::AboutDialog::builder()
            .application_name("App Icon Preview")
            .application_icon(config::APP_ID)
            .license_type(gtk::License::Gpl30)
            .website("https://gitlab.gnome.org/World/design/app-icon-preview/")
            .version(config::VERSION)
            .translator_credits(gettext("translator-credits"))
            .developers(vec!["Bilal Elmoussaoui", "Zander Brown"])
            .artists(vec!["Tobias Bernard"])
            .build()
            .present(&window);
    }

    pub fn icon_theme(&self) -> gtk::IconTheme {
        self.imp().icon_theme.get().unwrap().clone()
    }
}
