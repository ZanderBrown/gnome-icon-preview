use super::{ExportPopover, NewProjectDialog, ProjectPreviewer, RecentsPopover};
use crate::application::{Action, Application};
use crate::config::{APP_ID, PROFILE};
use crate::project::Project;

use gettextrs::gettext;
use log::error;
use std::rc::Rc;

use gtk::glib::clone;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gio, glib, CompositeTemplate};
use gtk_macros::{action, get_action, send};

#[derive(Debug, PartialEq)]
pub enum View {
    Initial,
    Previewer,
}

mod imp {
    use super::*;

    use once_cell::sync::OnceCell;
    use std::cell::RefCell;

    use adw::subclass::prelude::*;

    #[derive(CompositeTemplate)]
    #[template(resource = "/org/gnome/design/AppIconPreview/window.ui")]
    pub struct Window {
        pub sender: OnceCell<glib::Sender<Action>>,
        pub previewer: ProjectPreviewer,
        pub open_project: Rc<RefCell<Option<Project>>>,
        pub exporter: ExportPopover,
        pub monitor: RefCell<Option<gio::FileMonitor>>,
        pub settings: gio::Settings,

        #[template_child]
        pub content: TemplateChild<gtk::Stack>,
        #[template_child]
        pub export_btn: TemplateChild<gtk::MenuButton>,
        #[template_child]
        pub open_btn: TemplateChild<adw::SplitButton>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Window {
        const NAME: &'static str = "Window";
        type Type = super::Window;
        type ParentType = adw::ApplicationWindow;

        fn new() -> Self {
            Self {
                sender: OnceCell::new(),
                previewer: ProjectPreviewer::new(),
                open_project: Rc::new(RefCell::new(None)),
                exporter: ExportPopover::new(),
                monitor: RefCell::new(None),
                settings: gio::Settings::new(APP_ID),

                content: TemplateChild::default(),
                export_btn: TemplateChild::default(),
                open_btn: TemplateChild::default(),
            }
        }
        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }
        fn instance_init(obj: &gtk::glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }
    impl ObjectImpl for Window {
        fn constructed(&self, obj: &Self::Type) {
            let app = gio::Application::default().unwrap().downcast::<Application>().unwrap();

            self.sender.set(app.sender()).unwrap();

            if PROFILE == "Devel" {
                obj.add_css_class("devel");
            }

            obj.init();
            obj.setup_widgets();
            obj.setup_actions();
            obj.set_view(View::Initial);

            self.parent_constructed(obj);
        }
    }
    impl WidgetImpl for Window {}
    impl WindowImpl for Window {
        fn close_request(&self, window: &Self::Type) -> glib::signal::Inhibit {
            log::debug!("Saving window geometry.");

            let (width, height) = window.default_size();

            let _ = self.settings.set_int("window-width", width);
            let _ = self.settings.set_int("window-height", height);
            let _ = self.settings.set_boolean("is-maximized", window.is_maximized());

            self.parent_close_request(window)
        }
    }
    impl ApplicationWindowImpl for Window {}
    impl AdwApplicationWindowImpl for Window {}
}

glib::wrapper! {
    pub struct Window(ObjectSubclass<imp::Window>)
        @extends gtk::Widget, gtk::Window, gtk::ApplicationWindow, adw::ApplicationWindow, gio::ActionMap;
}

impl Window {
    pub fn new(app: &Application) -> Self {
        glib::Object::new(&[("application", app)]).unwrap()
    }

    pub fn set_open_project(&self, project: Project) {
        let self_ = imp::Window::from_instance(self);

        self.set_view(View::Previewer);
        self_.previewer.preview(&project);
        self_.exporter.set_project(&project);

        let recent_manager = gtk::RecentManager::default();
        recent_manager.add_item(&project.uri());

        let monitor = project.file().monitor_file(gio::FileMonitorFlags::all(), gio::NONE_CANCELLABLE).unwrap();

        self_.monitor.borrow_mut().replace(monitor);
        self_.open_project.borrow_mut().replace(project);

        self_.monitor.borrow().as_ref().unwrap().connect_changed(clone!(@strong self_.open_project as project,
        @strong self_.exporter as exporter, @strong self_.previewer as previewer  => move |monitor, _, _, event| {
            if event == gio::FileMonitorEvent::Changed {
                let file = project.borrow().as_ref().unwrap().file();
                match Project::parse(file, true) {
                    Ok(project) => {
                        monitor.cancel();
                        previewer.preview(&project);
                        exporter.set_project(&project);
                    }
                    Err(err) => log::warn!("Failed to parse the project {}", err),
                }
            }
        }));
    }

    pub fn set_view(&self, view: View) {
        let self_ = imp::Window::from_instance(self);

        get_action!(self, @shuffle).set_enabled(view == View::Previewer);
        get_action!(self, @refresh).set_enabled(view == View::Previewer);
        get_action!(self, @save_screenshot).set_enabled(view == View::Previewer);
        get_action!(self, @copy_screenshot).set_enabled(view == View::Previewer);

        match view {
            View::Previewer => {
                self_.content.set_visible_child_name("previewer");
                self_.export_btn.show();
            }
            View::Initial => {
                self_.export_btn.hide();
            }
        };
    }

    fn setup_widgets(&self) {
        let self_ = imp::Window::from_instance(self);

        self_.content.add_named(&self_.previewer, Some("previewer"));

        // Recents Popover
        let recents_popover = RecentsPopover::new(self_.sender.get().unwrap().clone());
        self_.open_btn.set_popover(Some(&recents_popover));

        // TODO
        // self_.exporter.widget.set_relative_to(Some(&export_btn));
        self_.export_btn.set_popover(Some(&self_.exporter));
    }

    fn setup_actions(&self) {
        let self_ = imp::Window::from_instance(self);
        let sender = self_.sender.get().unwrap().clone();

        // Export icon
        action!(
            self,
            "export",
            clone!(@strong self_.exporter as exporter => move |_, _| {
                exporter.popup();
            })
        );

        action!(
            self,
            "export-save",
            Some(glib::VariantTy::new("s").unwrap()),
            clone!(@weak self_.open_project as project, @weak self as parent => move |_, target| {
                if let Some(project) = project.borrow().as_ref() {
                    let project_type = target.unwrap().get::<String>().unwrap();
                    let icon = crate::common::Icon::from(project_type);
                    let fut = clone!(@weak project, @weak parent => async move {
                        if project.export(icon, &parent.upcast::<gtk::Window>()).await.is_err() {
                            log::warn!("Failed to export the project");
                        }
                    });
                    gtk_macros::spawn!(fut);
                };
            })
        );

        // New Project
        action!(
            self,
            "new-project",
            clone!(@weak self as window, @strong sender => move |_, _| {
                let dialog = NewProjectDialog::new(sender.clone());
                dialog.set_transient_for(Some(&window));
                dialog.show();
            })
        );

        // Refresh
        action!(
            self,
            "refresh",
            clone!(@strong sender, @weak self_.open_project as project,
            @strong self_.exporter as exporter, @strong self_.previewer as previewer => move |_, _| {
                if let Some(project) = project.borrow().as_ref() {
                   match Project::parse(project.file(), true) {
                        Ok(project) => {
                            previewer.preview(&project);
                            exporter.set_project(&project);
                        },
                        Err(err) => log::warn!("Failed to parse the project {}", err),
                    }
                };
            })
        );

        // Shuffle sample icons
        action!(
            self,
            "shuffle",
            clone!(@strong self_.previewer as previewer => move |_, _| {
                previewer.shuffle_samples();
            })
        );

        // Save Screenshot
        action!(
            self,
            "save_screenshot",
            clone!(@weak self as window, @strong self_.previewer as previewer => move |_, _| {
                previewer.save_screenshot();
            })
        );

        // Copy Screenshot
        action!(
            self,
            "copy_screenshot",
            clone!(@strong self_.previewer as previewer => move |_, _| {
                previewer.copy_screenshot();
            })
        );

        // Open file
        action!(
            self,
            "open",
            clone!(@weak self as window, @strong sender => move |_, _| {
                let file_chooser = gtk::FileChooserNative::new(Some(&gettext("Open File")),
                                        Some(&window), gtk::FileChooserAction::Open,
                                        None, None);

                let svg_filter = gtk::FileFilter::new();
                svg_filter.set_name(Some(&gettext("SVG images")));
                svg_filter.add_mime_type("image/svg+xml");

                file_chooser.add_filter(&svg_filter);

                file_chooser.connect_response(clone!(@strong file_chooser, @strong sender => move |_, response| {
                    if response == gtk::ResponseType::Accept {
                        let file = file_chooser.file().unwrap();
                        match Project::parse(file, true) {
                            Ok(project) => send!(sender, Action::OpenProject(project)),
                            Err(err) => log::warn!("Failed to open file {}", err),
                        };
                    file_chooser.destroy();
                }}));
                file_chooser.show()
            })
        );
    }

    fn init(&self) {
        // load latest window state
        let self_ = imp::Window::from_instance(self);

        let width = self_.settings.int("window-width");
        let height = self_.settings.int("window-height");
        if width > -1 && height > -1 {
            self.set_default_size(width, height);
        }
        let is_maximized = self_.settings.boolean("is-maximized");

        if is_maximized {
            self.maximize();
        }
    }
}
