use adw::{prelude::*, subclass::prelude::*};
use gettextrs::gettext;
use gtk::{gdk, gio, glib, glib::clone};

use super::{ExportPopover, ProjectPreviewer};
use crate::{
    application::Application,
    config::{APP_ID, PROFILE},
    project::Project,
};

#[derive(Debug, Eq, PartialEq)]
pub enum View {
    Initial,
    Previewer,
}

mod imp {
    use std::cell::RefCell;

    use super::*;
    use crate::widgets::RecentsPopover;

    #[derive(gtk::CompositeTemplate)]
    #[template(resource = "/org/gnome/design/AppIconPreview/window.ui")]
    pub struct Window {
        pub previewer: ProjectPreviewer,
        pub open_project: RefCell<Option<Project>>,
        pub exporter: ExportPopover,
        pub monitor: RefCell<Option<gio::FileMonitor>>,
        pub settings: gio::Settings,

        #[template_child]
        pub content: TemplateChild<gtk::Stack>,
        #[template_child]
        pub export_btn: TemplateChild<gtk::MenuButton>,
        #[template_child]
        pub open_btn: TemplateChild<adw::SplitButton>,
        #[template_child]
        pub toolbar_view: TemplateChild<adw::ToolbarView>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Window {
        const NAME: &'static str = "Window";
        type Type = super::Window;
        type ParentType = adw::ApplicationWindow;

        fn new() -> Self {
            Self {
                previewer: Default::default(),
                open_project: Default::default(),
                exporter: Default::default(),
                monitor: Default::default(),
                settings: gio::Settings::new(APP_ID),
                content: Default::default(),
                export_btn: Default::default(),
                open_btn: Default::default(),
                toolbar_view: Default::default(),
            }
        }
        fn class_init(klass: &mut Self::Class) {
            RecentsPopover::ensure_type();

            klass.bind_template();
            klass.bind_template_instance_callbacks();

            // Export icon
            klass.install_action("win.export", None, |window, _, _| {
                window.imp().exporter.popup();
            });

            klass.install_action_async(
                "win.export-save",
                Some(glib::VariantTy::STRING),
                |window, _, target| async move {
                    if let Some(project) = window.imp().open_project.borrow().as_ref() {
                        let project_type = target.unwrap().get::<String>().unwrap();
                        let icon = crate::common::Icon::from(project_type);
                        if project.export(icon, &window).await.is_err() {
                            log::warn!("Failed to export the project");
                        }
                    };
                },
            );

            // New Project
            klass.install_action_async("win.new-project", None, |window, _, _| async move {
                if let Err(err) = window.new_project().await {
                    log::warn!("Failed to create a new project {err}");
                }
            });

            // Refresh
            klass.install_action("win.refresh", None, |window, _, _| {
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
            klass.install_action("win.shuffle", None, |window, _, _| {
                window.imp().previewer.shuffle_samples();
            });

            // Save Screenshot
            klass.install_action_async("win.save-screenshot", None, |window, _, _| async move {
                window
                    .imp()
                    .previewer
                    .save_screenshot()
                    .await
                    .unwrap_or_else(|err| log::error!("Could not save screenshot: {}", err));
            });

            // Copy Screenshot
            klass.install_action("win.copy-screenshot", None, |window, _, _| {
                window.imp().previewer.copy_screenshot();
            });

            // Open file
            klass.install_action_async("win.open", None, |window, _, _| async move {
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
        @extends gtk::Widget, gtk::Window, gtk::ApplicationWindow, adw::ApplicationWindow, gio::ActionMap,
        @implements gtk::Root;
}

#[gtk::template_callbacks]
impl Window {
    pub fn new(app: &Application) -> Self {
        glib::Object::builder().property("application", app).build()
    }

    #[template_callback]
    fn on_recent_selected(&self, project: Project) {
        self.set_open_project(project);
    }

    pub fn set_open_project(&self, project: Project) {
        let imp = self.imp();

        self.set_view(View::Previewer);
        imp.previewer.preview(&project);
        imp.exporter.set_project(&project);

        let recent_manager = gtk::RecentManager::default();
        recent_manager.add_item(&project.uri());

        let monitor = project
            .file()
            .monitor_file(gio::FileMonitorFlags::all(), gio::Cancellable::NONE)
            .unwrap();

        imp.monitor.borrow_mut().replace(monitor);
        imp.open_project.borrow_mut().replace(project);

        imp.monitor.borrow().as_ref().unwrap().connect_changed(
            clone!(@weak self as this => move |monitor, _, _, event| {
                if event == gio::FileMonitorEvent::Changed {
                    let project = &this.imp().open_project;
                    let file = project.borrow().as_ref().unwrap().file();
                    match Project::parse(file, true) {
                        Ok(project) => {
                            monitor.cancel();
                            this.set_open_project(project);
                        }
                        Err(err) => log::warn!("Failed to parse the project {}", err),
                    }
                }
            }),
        );
    }

    async fn new_project(&self) -> anyhow::Result<()> {
        let dialog = gtk::FileDialog::builder()
            .accept_label(gettext("_Create"))
            .initial_name("com.domain.Application.svg")
            .modal(true)
            .title(gettext("Select a file"))
            .build();
        let file = dialog.save_future(Some(self)).await?;
        let project = Project::from_template(file)?;
        self.set_open_project(project);
        Ok(())
    }

    async fn open_file(&self) -> anyhow::Result<()> {
        let filters = gio::ListStore::new::<gtk::FileFilter>();
        let svg_filter = gtk::FileFilter::new();
        svg_filter.set_name(Some(&gettext("SVG images")));
        svg_filter.add_mime_type("image/svg+xml");
        filters.append(&svg_filter);

        let dialog = gtk::FileDialog::builder()
            .title(gettext("Open File"))
            .modal(true)
            .filters(&filters)
            .build();

        let file = dialog.open_future(Some(self)).await?;
        let project = Project::parse(file, true)?;

        self.set_open_project(project);
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
                imp.toolbar_view
                    .set_top_bar_style(adw::ToolbarStyle::Raised);
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
        imp.export_btn.set_popover(Some(&imp.exporter));
    }

    fn setup_drop(&self) {
        let target = gtk::DropTarget::new(
            gio::File::static_type(),
            gdk::DragAction::COPY | gdk::DragAction::MOVE,
        );

        target.connect_drop(
            glib::clone!(@weak self as obj => @default-return false, move |_, value, _, _| {
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
            }),
        );

        self.add_controller(target);
    }
}
