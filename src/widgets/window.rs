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
use gtk::{gdk, gio, glib, CompositeTemplate};
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
            self.parent_constructed(obj);

            let app = gio::Application::default().unwrap().downcast::<Application>().unwrap();

            self.sender.set(app.sender()).unwrap();

            if PROFILE == "Devel" {
                obj.add_css_class("devel");
            }

            obj.setup_widgets();
            obj.setup_actions();
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
        glib::Object::new(&[("application", app)]).unwrap()
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

        imp.monitor.borrow().as_ref().unwrap().connect_changed(clone!(@strong imp.open_project as project,
        @strong imp.exporter as exporter, @strong imp.previewer as previewer  => move |monitor, _, _, event| {
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
        let imp = self.imp();

        get_action!(self, @shuffle).set_enabled(view == View::Previewer);
        get_action!(self, @refresh).set_enabled(view == View::Previewer);
        get_action!(self, @save_screenshot).set_enabled(view == View::Previewer);
        get_action!(self, @copy_screenshot).set_enabled(view == View::Previewer);

        match view {
            View::Previewer => {
                imp.content.set_visible_child_name("previewer");
                imp.export_btn.show();
            }
            View::Initial => {
                imp.export_btn.hide();
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

    fn setup_actions(&self) {
        let imp = self.imp();
        let sender = imp.sender.get().unwrap().clone();

        // Export icon
        action!(
            self,
            "export",
            clone!(@strong imp.exporter as exporter => move |_, _| {
                exporter.popup();
            })
        );

        action!(
            self,
            "export-save",
            Some(glib::VariantTy::new("s").unwrap()),
            clone!(@weak imp.open_project as project, @weak self as parent => move |_, target| {
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
            clone!(@strong sender, @weak imp.open_project as project,
            @strong imp.exporter as exporter, @strong imp.previewer as previewer => move |_, _| {
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
            clone!(@strong imp.previewer as previewer => move |_, _| {
                previewer.shuffle_samples();
            })
        );

        // Save Screenshot
        action!(
            self,
            "save_screenshot",
            clone!(@strong imp.previewer as previewer => move |_, _| {
                let ctx = glib::MainContext::default();
                ctx.spawn_local(clone!(@weak previewer => async move {
                    previewer.save_screenshot()
                             .await
                             .unwrap_or_else(|err| log::error!("Could not save screenshot: {}", err));
                }));
            })
        );

        // Copy Screenshot
        action!(
            self,
            "copy_screenshot",
            clone!(@strong imp.previewer as previewer => move |_, _| {
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

        self.add_controller(&target);
    }
}
