use adw::{prelude::*, subclass::prelude::*};
use gettextrs::gettext;
use gtk::{gio, glib};

use crate::{config, project::Project, widgets::Window};

mod imp {
    use super::*;

    #[derive(Default)]
    pub struct Application {}

    #[glib::object_subclass]
    impl ObjectSubclass for Application {
        const NAME: &'static str = "Application";
        type ParentType = adw::Application;
        type Type = super::Application;
    }

    impl ObjectImpl for Application {
        fn constructed(&self) {
            self.parent_constructed();

            self.obj().add_main_option(
                "new-window",
                glib::Char::from(b'w'),
                glib::OptionFlags::NONE,
                glib::OptionArg::None,
                &gettext("Open a new window"),
                None,
            );
        }
    }

    impl ApplicationImpl for Application {
        fn handle_local_options(&self, options: &glib::VariantDict) -> glib::ExitCode {
            let app = self.obj();

            if options.contains("new-window") {
                if let Err(err) = app.register(None::<&gio::Cancellable>) {
                    log::error!("Failed to register the application: {err}");
                }

                if app.is_remote() {
                    app.activate_action("new-window", None);
                    return glib::ExitCode::SUCCESS;
                }
            }

            self.parent_handle_local_options(options)
        }

        fn startup(&self) {
            self.parent_startup();
            // setup icon theme cache
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

            if let Some(window) = application.active_window() {
                window.present();
                return;
            }

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
        @extends gio::Application, gtk::Application, adw::Application,
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
            .application_name(gettext("App Icon Preview"))
            .application_icon(config::APP_ID)
            .license_type(gtk::License::Gpl30)
            .website("https://gitlab.gnome.org/World/design/app-icon-preview/")
            .version(config::VERSION)
            .translator_credits(gettext("translator-credits"))
            .developers(vec!["Bilal Elmoussaoui", "Zander Brown"])
            .artists(vec!["Tobias Bernard"])
            .build()
            .present(Some(&window));
    }
}
