use crate::config;
use crate::project::Project;
use crate::widgets::Window;

use adw::prelude::*;
use gettextrs::gettext;
use gtk::glib::{clone, Receiver, Sender};
use gtk::{gdk, gio, glib, subclass::prelude::*};
use log::error;

pub enum Action {
    OpenProject(Project),
    NewProject(gio::File),
}

mod imp {
    use super::*;
    use adw::subclass::prelude::*;
    use once_cell::sync::OnceCell;
    use std::cell::RefCell;

    pub struct Application {
        pub sender: Sender<Action>,
        pub receiver: RefCell<Option<Receiver<Action>>>,
        pub icon_theme: OnceCell<gtk::IconTheme>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Application {
        const NAME: &'static str = "Application";
        type ParentType = adw::Application;
        type Type = super::Application;

        fn new() -> Self {
            let (sender, r) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);
            let receiver = RefCell::new(Some(r));

            Self {
                sender,
                receiver,
                icon_theme: OnceCell::new(),
            }
        }
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
            let quit = gio::ActionEntry::builder("quit").activate(move |app: &Self::Type, _, _| app.quit()).build();

            let about = gio::ActionEntry::builder("about").activate(move |app: &Self::Type, _, _| app.show_about_dialog()).build();

            app.add_action_entries([new_window, quit, about]).unwrap();

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
            let application = self.instance();
            let window = application.create_window();
            window.present();
            // Setup action channel
            let receiver = self.receiver.borrow_mut().take().unwrap();
            receiver.attach(None, clone!(@strong application => move |action| application.do_action(action)));
        }

        fn open(&self, files: &[gio::File], _hint: &str) {
            let application = self.instance();
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

        let app = glib::Object::new::<Self>(&[
            ("application-id", &config::APP_ID),
            ("flags", &gio::ApplicationFlags::HANDLES_OPEN),
            ("resource-base-path", &Some("/org/gnome/design/AppIconPreview")),
        ]);
        app.run();
    }

    fn create_window(&self) -> Window {
        let group = gtk::WindowGroup::new();
        let window = Window::new(self);

        group.add_window(&window);

        window
    }

    fn do_action(&self, action: Action) -> glib::Continue {
        match action {
            Action::OpenProject(project) => {
                let window = self.active_window().unwrap().downcast::<Window>().unwrap();
                window.set_open_project(project);
            }
            Action::NewProject(project_dest) => match Project::from_template(project_dest) {
                Ok(project) => self.imp().sender.send(Action::OpenProject(project)).unwrap(),
                Err(err) => error!("{:#?}", err),
            },
        };
        glib::Continue(true)
    }

    fn show_about_dialog(&self) {
        let window = self.active_window().unwrap().downcast::<Window>().unwrap();
        adw::AboutWindow::builder()
            .application_name("App Icon Preview")
            .application_icon(config::APP_ID)
            .license_type(gtk::License::Gpl30)
            .website("https://gitlab.gnome.org/World/design/app-icon-preview/")
            .version(config::VERSION)
            .transient_for(&window)
            .translator_credits(&gettext("translator-credits"))
            .modal(true)
            .developers(vec!["Bilal Elmoussaoui".into(), "Zander Brown".into()])
            .artists(vec!["Tobias Bernard".into()])
            .build()
            .present();
    }

    pub fn sender(&self) -> Sender<Action> {
        self.imp().sender.clone()
    }

    pub fn icon_theme(&self) -> gtk::IconTheme {
        self.imp().icon_theme.get().unwrap().clone()
    }
}
