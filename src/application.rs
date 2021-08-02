use crate::config;
use crate::project::Project;
use crate::widgets::Window;

use log::error;
use std::rc::Rc;

use gtk::glib::{clone, Receiver, Sender};
use gtk::{gdk, gio, glib, prelude::*, subclass::prelude::*};
use gtk_macros::{action, send};

pub enum Action {
    OpenProject(Rc<Project>),
    NewProject(gio::File),
}

mod imp {
    use super::*;

    use std::cell::RefCell;

    pub struct Application {
        pub windows: gtk::WindowGroup,
        pub sender: Sender<Action>,
        pub receiver: RefCell<Option<Receiver<Action>>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Application {
        const NAME: &'static str = "Application";
        type ParentType = gtk::Application;
        type Type = super::Application;

        fn new() -> Self {
            let (sender, r) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);
            let receiver = RefCell::new(Some(r));

            Self {
                windows: gtk::WindowGroup::new(),
                sender,
                receiver,
            }
        }
    }
    impl ObjectImpl for Application {}
    impl GtkApplicationImpl for Application {
        fn window_removed(&self, application: &Self::Type, window: &gtk::Window) {
            self.parent_window_removed(application, window);
            self.windows.remove_window(window);
        }
    }
    impl ApplicationImpl for Application {
        fn startup(&self, application: &Self::Type) {
            self.parent_startup(application);

            adw::init();

            // setup css
            let p = gtk::CssProvider::new();
            gtk::CssProvider::load_from_resource(&p, "/org/gnome/design/AppIconPreview/style.css");
            if let Some(display) = gdk::Display::default() {
                gtk::StyleContext::add_provider_for_display(&display, &p, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);
            }

            action!(
                application,
                "new-window",
                clone!(@weak application => move |_, _| {
                    let window = application.create_window();
                    window.present();
                })
            );

            action!(
                application,
                "quit",
                clone!(@weak application => move |_, _| {
                    application.quit();
                })
            );
        }
        fn activate(&self, application: &Self::Type) {
            let window = application.create_window();
            window.present();

            // Accelerators
            application.set_accels_for_action("win.open", &["<primary>o"]);
            application.set_accels_for_action("win.refresh", &["<primary>r"]);
            application.set_accels_for_action("win.shuffle", &["<primary>s"]);
            application.set_accels_for_action("win.export", &["<primary>e"]);
            application.set_accels_for_action("win.screenshot", &["<primary><alt>s"]);
            application.set_accels_for_action("win.copy-screenshot", &["<primary><alt>c"]);
            application.set_accels_for_action("app.quit", &["<primary>q"]);
            application.set_accels_for_action("app.new-window", &["<primary>n"]);

            // Setup action channel
            let receiver = self.receiver.borrow_mut().take().unwrap();
            receiver.attach(None, clone!(@strong application => move |action| application.do_action(action)));
        }

        fn open(&self, application: &Self::Type, files: &[gio::File], _hint: &str) {
            for file in files.iter() {
                if let Ok(project) = Project::parse(file.clone()) {
                    let window = application.create_window();
                    window.set_open_project(project);
                    window.present();
                }
            }
        }
    }
}

glib::wrapper! {
    pub struct Application(ObjectSubclass<imp::Application>) @extends gio::Application, gtk::Application, gio::ActionMap;
}

impl Application {
    pub fn run() {
        log::info!("App Icon Preview ({})", config::APP_ID);
        log::info!("Version: {} ({})", config::VERSION, config::PROFILE);
        log::info!("Datadir: {}", config::PKGDATADIR);

        let app = glib::Object::new::<Self>(&[
            ("application-id", &config::APP_ID),
            ("flags", &gio::ApplicationFlags::FLAGS_NONE),
            ("resource-base-path", &Some("/org/gnome/design/AppIconPreview")),
        ])
        .unwrap();

        ApplicationExtManual::run(&app);
    }

    fn create_window(&self) -> Window {
        let self_ = imp::Application::from_instance(self);
        let window = Window::new(self_.sender.clone(), self);

        self_.windows.add_window(&window);
        window
    }

    fn do_action(&self, action: Action) -> glib::Continue {
        let self_ = imp::Application::from_instance(self);

        match action {
            Action::OpenProject(project) => {
                let window = self.active_window().unwrap().downcast::<Window>().unwrap();
                window.set_open_project(project);
            }
            Action::NewProject(project_dest) => match Project::from_template(project_dest) {
                Ok(project) => send!(self_.sender, Action::OpenProject(project)),
                Err(err) => error!("{:#?}", err),
            },
        };
        glib::Continue(true)
    }
}
