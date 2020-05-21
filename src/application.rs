use crate::project::Project;
use gio::prelude::*;
use glib::{Receiver, Sender};
use gtk::prelude::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::env;
use std::rc::Rc;

use crate::config;
use crate::recents::RecentManager;
use crate::widgets::Window;

pub enum Action {
    OpenProject(Project),
    NewProject(gio::File),
}

pub struct Application {
    app: gtk::Application,
    windows: Rc<RefCell<HashMap<gtk::Window, Rc<Window>>>>,
    sender: Sender<Action>,
    receiver: RefCell<Option<Receiver<Action>>>,
    history: RecentManager,
}

impl Application {
    pub fn new() -> Rc<Self> {
        let app = gtk::Application::new(Some(config::APP_ID), gio::ApplicationFlags::FLAGS_NONE).unwrap();

        let (sender, r) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);
        let receiver = RefCell::new(Some(r));

        let application = Rc::new(Self {
            app,
            windows: Rc::new(RefCell::new(HashMap::new())),
            sender,
            receiver,
            history: RecentManager::new(),
        });

        application.setup_signals(application.clone());
        application.setup_css();
        application
    }

    fn get_window(&self) -> Rc<Window> {
        let gtk_window = self.app.get_active_window().expect("Failed to get a GtkWindow");
        self.windows.borrow().get(&gtk_window).expect("Failed to get a Window").clone()
    }

    fn create_window(&self) -> Rc<Window> {
        let window = Window::new(&self.history, self.sender.clone());

        window.widget.set_application(Some(&self.app));
        self.app.add_window(&window.widget);

        let gtk_window = window.widget.clone();

        // TODO: fix this once we have subclasses
        self.windows.borrow_mut().insert(gtk_window.upcast::<gtk::Window>(), window.clone());
        window
    }

    fn setup_gactions(&self, app: Rc<Self>) {
        action!(
            self.app,
            "new-window",
            clone!(@weak app => move |_, _| {
                let window = app.create_window();
                window.widget.present();
            })
        );

        // Quit
        action!(
            self.app,
            "quit",
            clone!(@weak app => move |_, _| {
               app.get_window().widget.destroy();
            })
        );
        self.app.set_accels_for_action("win.open", &["<primary>o"]);
        self.app.set_accels_for_action("win.refresh", &["<primary>r"]);
        self.app.set_accels_for_action("win.shuffle", &["<primary>s"]);
        self.app.set_accels_for_action("win.export", &["<primary>e"]);
        self.app.set_accels_for_action("win.screenshot", &["<primary><alt>s"]);
        self.app.set_accels_for_action("win.copy-screenshot", &["<primary><alt>c"]);
        self.app.set_accels_for_action("win.show-help-overlay", &["<primary>comma"]);
        self.app.set_accels_for_action("app.quit", &["<primary>q"]);
        self.app.set_accels_for_action("app.new-window", &["<primary>n"]);
    }

    fn setup_signals(&self, app: Rc<Self>) {
        self.app.connect_activate(clone!(@weak app => move |_| {
            app.get_window().widget.present();
        }));

        self.app.connect_window_removed(clone!(@weak app => move |_, window| {
            app.windows.borrow_mut().remove(window);
        }));

        self.app.connect_startup(clone!(@weak app => move |_| {
            app.create_window();
            app.setup_gactions(app.clone());
        }));

        self.app.connect_open(clone!(@weak app => move |_, files, _| {
            for file in files.iter() {
                if let Ok(project) = Project::parse(file.clone()) {
                    let window = app.create_window();
                    window.set_open_project(project);
                    window.widget.present();
                }
            }
        }));
    }

    fn setup_css(&self) {
        self.app.set_resource_base_path(Some("/org/gnome/design/AppIconPreview/"));

        let p = gtk::CssProvider::new();
        gtk::CssProvider::load_from_resource(&p, "/org/gnome/design/AppIconPreview/style.css");
        if let Some(screen) = gdk::Screen::get_default() {
            gtk::StyleContext::add_provider_for_screen(&screen, &p, 500);
        }
        if let Some(theme) = gtk::IconTheme::get_default() {
            theme.add_resource_path("/org/gnome/design/AppIconPreview/icons");
        }
    }

    fn do_action(&self, action: Action) -> glib::Continue {
        match action {
            Action::OpenProject(project) => {
                self.history.add(project.path());
                self.get_window().set_open_project(project);
            }
            Action::NewProject(project_dest) => match Project::from_template(project_dest) {
                Ok(project) => send!(self.sender, Action::OpenProject(project)),
                Err(err) => println!("{:#?}", err),
            },
        };
        glib::Continue(true)
    }

    pub fn run(&self, app: Rc<Self>) {
        info!("App Icon Preview{} ({})", config::NAME_SUFFIX, config::APP_ID);
        info!("Version: {} ({})", config::VERSION, config::PROFILE);
        info!("Datadir: {}", config::PKGDATADIR);

        let receiver = self.receiver.borrow_mut().take().unwrap();
        receiver.attach(None, move |action| app.do_action(action));

        let args: Vec<String> = env::args().collect();
        self.app.run(&args);
    }
}
