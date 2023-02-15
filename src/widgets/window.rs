use super::{ExportPopover, NewProjectDialog, ProjectPreviewer, RecentsPopover};
use crate::application::{Action, Application};
use crate::config::{APP_ID, PROFILE};
use crate::project::Project;

use gettextrs::gettext;
use std::rc::Rc;

use gtk::glib::clone;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gdk, gio, glib};

#[derive(Debug, Eq, PartialEq)]
pub enum View {
    Initial,
    Previewer,
}

mod imp {
    use super::*;

    use once_cell::sync::OnceCell;
    use std::cell::RefCell;

    use adw::subclass::prelude::*;

    #[derive(gtk::CompositeTemplate)]
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
            klass.bind_template();

            // Export icon
            klass.install_action("win.export", None, move |window, _, _| {
                window.imp().exporter.popup();
            });

            klass.install_action_async("win.export-save", Some("s"), move |window, _, target| async move {
                if let Some(project) = window.imp().open_project.borrow().as_ref() {
                    let project_type = target.unwrap().get::<String>().unwrap();
                    let icon = crate::common::Icon::from(project_type);
                    if project.export(icon, &window).await.is_err() {
                        log::warn!("Failed to export the project");
                    }
                };
            });

            // New Project
            klass.install_action("win.new-project", None, move |window, _, _| {
                let sender = window.imp().sender.get().unwrap().clone();
                let dialog = NewProjectDialog::new(sender);
                dialog.set_transient_for(Some(window));
                dialog.present();
            });

            // Refresh
            klass.install_action("win.refresh", None, move |window, _, _| {
                let imp = window.imp();
                if let Some(project) = imp.open_project.borrow().as_ref() {
                    match Project::parse(project.file(), true) {
                        Ok(project) => {
                            imp.previewer.preview(&project);
                            imp.exporter.set_project(&project);
                        }
                        Err(err) => log::warn!("Failed to parse the project {}", err),
                    }
                };
            });

            // Shuffle sample icons
            klass.install_action("win.shuffle", None, move |window, _, _| {
                window.imp().previewer.shuffle_samples();
            });

            // Save Screenshot
            klass.install_action_async("win.save-screenshot", None, move |window, _, _| async move {
                window.imp().previewer.save_screenshot().await.unwrap_or_else(|err| log::error!("Could not save screenshot: {}", err));
            });

            // Copy Screenshot
            klass.install_action("win.copy-screenshot", None, move |window, _, _| {
                window.imp().previewer.copy_screenshot();
            });

            // Open file
            klass.install_action_async("win.open", None, move |window, _, _| async move {
                if let Err(err) = window.open_file().await {
                    log::warn!("Failed to open file {err}");
                }
            });
        }
        fn instance_init(obj: &gtk::glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }
    impl ObjectImpl for Window {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();
            let app = gio::Application::default().unwrap().downcast::<Application>().unwrap();

            self.sender.set(app.sender()).unwrap();

            if PROFILE == "Devel" {
                obj.add_css_class("devel");
            }

            obj.setup_widgets();
            obj.set_view(View::Initial);
            obj.setup_drop();
        }
    }
    impl WidgetImpl for Window {}
    impl WindowImpl for Window {}
    impl ApplicationWindowImpl for Window {}
    impl AdwApplicationWindowImpl for Window {}
}

glib::wrapper! {
    pub struct Window(ObjectSubclass<imp::Window>)
        @extends gtk::Widget, gtk::Window, gtk::ApplicationWindow, adw::ApplicationWindow, gio::ActionMap;
}

impl Window {
    pub fn new(app: &Application) -> Self {
        glib::Object::builder().property("application", app).build()
    }

    pub fn set_open_project(&self, project: Project) {
        let imp = self.imp();

        self.set_view(View::Previewer);
        imp.previewer.preview(&project);
        imp.exporter.set_project(&project);

        let recent_manager = gtk::RecentManager::default();
        recent_manager.add_item(&project.uri());

        let monitor = project.file().monitor_file(gio::FileMonitorFlags::all(), gio::Cancellable::NONE).unwrap();

        imp.monitor.borrow_mut().replace(monitor);
        imp.open_project.borrow_mut().replace(project);

        imp.monitor.borrow().as_ref().unwrap().connect_changed(clone!(@weak imp.open_project as project,
        @weak self as this => move |monitor, _, _, event| {
            if event == gio::FileMonitorEvent::Changed {
                let file = project.borrow().as_ref().unwrap().file();
                match Project::parse(file, true) {
                    Ok(project) => {
                        monitor.cancel();
                        this.set_open_project(project);
                    }
                    Err(err) => log::warn!("Failed to parse the project {}", err),
                }
            }
        }));
    }

    async fn open_file(&self) -> anyhow::Result<()> {
        let filters = gio::ListStore::new(gtk::FileFilter::static_type());
        let svg_filter = gtk::FileFilter::new();
        svg_filter.set_name(Some(&gettext("SVG images")));
        svg_filter.add_mime_type("image/svg+xml");
        filters.append(&svg_filter);

        let dialog = gtk::FileDialog::builder().title(gettext("Open File")).modal(true).filters(&filters).build();

        let file = dialog.open_future(Some(self)).await?;
        let sender = self.imp().sender.get().unwrap();
        let project = Project::parse(file, true)?;
        sender.send(Action::OpenProject(project)).unwrap();
        Ok(())
    }

    pub fn set_view(&self, view: View) {
        let imp = self.imp();

        self.action_set_enabled("win.shuffle", view == View::Previewer);
        self.action_set_enabled("win.refresh", view == View::Previewer);
        self.action_set_enabled("win.save-screenshot", view == View::Previewer);
        self.action_set_enabled("win.copy-screenshot", view == View::Previewer);

        match view {
            View::Previewer => {
                imp.content.set_visible_child_name("previewer");
                imp.export_btn.set_visible(true);
            }
            View::Initial => {
                imp.export_btn.set_visible(false);
            }
        };
    }

    fn setup_widgets(&self) {
        let imp = self.imp();

        imp.content.add_named(&imp.previewer, Some("previewer"));

        // Recents Popover
        let recents_popover = RecentsPopover::new(imp.sender.get().unwrap().clone());
        imp.open_btn.set_popover(Some(&recents_popover));

        imp.export_btn.set_popover(Some(&imp.exporter));
    }

    fn setup_drop(&self) {
        let target = gtk::DropTarget::new(gio::File::static_type(), gdk::DragAction::COPY | gdk::DragAction::MOVE);

        target.connect_drop(glib::clone!(@weak self as obj => @default-return false, move |_, value, _, _| {
            if let Ok(file) = value.get::<gio::File>() {
                match Project::parse(file, true) {
                    Ok(project) => {
                        obj.set_open_project(project);
                        return true
                    },
                    Err(err) => log::warn!("Failed to parse the project {}", err),
                }
            }

            false
        }));

        self.add_controller(target);
    }
}
