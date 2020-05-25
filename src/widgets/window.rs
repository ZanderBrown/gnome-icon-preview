use super::{ExportPopover, NewProjectDialog, ProjectPreviewer, RecentsPopover, ScreenshotDialog};
use crate::application::Action;
use crate::config::PROFILE;
use crate::project::Project;
use crate::recents::RecentManager;
use crate::settings::{Key, SettingsManager};

use gettextrs::gettext;
use gio::prelude::*;
use gtk::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug, PartialEq)]
pub enum View {
    Initial,
    Previewer,
}

pub struct Window {
    pub widget: gtk::ApplicationWindow,
    builder: gtk::Builder,
    sender: glib::Sender<Action>,
    previewer: ProjectPreviewer,
    open_project: Rc<RefCell<Option<Project>>>,
    exporter: ExportPopover,
    monitor: RefCell<Option<gio::FileMonitor>>,
}

impl Window {
    pub fn new(history: &RecentManager, sender: glib::Sender<Action>) -> Rc<Self> {
        let builder = gtk::Builder::new_from_resource("/org/gnome/design/AppIconPreview/window.ui");
        get_widget!(builder, gtk::ApplicationWindow, window);
        let previewer = ProjectPreviewer::new();

        let window_widget = Rc::new(Window {
            widget: window,
            builder,
            sender,
            previewer,
            exporter: ExportPopover::new(),
            open_project: Rc::new(RefCell::new(None)),
            monitor: RefCell::new(None),
        });

        window_widget.init();
        window_widget.setup_widgets(history);
        window_widget.setup_actions();
        window_widget.set_view(View::Initial);
        window_widget
    }

    pub fn set_open_project(&self, project: Project) {
        self.set_view(View::Previewer);
        self.previewer.preview(&project);
        self.exporter.set_project(&project);

        let monitor = project.file.monitor_file(gio::FileMonitorFlags::all(), gio::NONE_CANCELLABLE).unwrap();

        self.monitor.borrow_mut().replace(monitor);
        self.open_project.borrow_mut().replace(project.clone());

        self.monitor
            .borrow()
            .as_ref()
            .unwrap()
            .connect_changed(clone!(@strong self.sender as sender => move |monitor, _, _, event| {
                if event == gio::FileMonitorEvent::Changed {
                    match Project::parse(project.file.clone()) {
                        Ok(project) => {
                            monitor.cancel();
                            send!(sender, Action::OpenProject(project));
                        }
                        Err(err) => warn!("Failed to parse the project {}", err),
                    }
                }
            }));
    }

    pub fn set_view(&self, view: View) {
        get_widget!(self.builder, gtk::Stack, content);
        get_widget!(self.builder, gtk::MenuButton, export_btn);

        get_action!(self.widget, @shuffle).set_enabled(view == View::Previewer);
        get_action!(self.widget, @refresh).set_enabled(view == View::Previewer);
        get_action!(self.widget, @screenshot).set_enabled(view == View::Previewer);

        match view {
            View::Previewer => {
                content.set_visible_child_name("previewer");
                export_btn.show();
            }
            View::Initial => {
                export_btn.hide();
            }
        };
    }

    fn setup_widgets(&self, history: &RecentManager) {
        get_widget!(self.builder, gtk::MenuButton, open_menu_btn);

        let builder = gtk::Builder::new_from_resource("/org/gnome/design/AppIconPreview/help-overlay.ui");
        get_widget!(builder, gtk::ShortcutsWindow, help_overlay);
        self.widget.set_help_overlay(Some(&help_overlay));

        let menu_builder = gtk::Builder::new_from_resource("/org/gnome/design/AppIconPreview/menus.ui");
        get_widget!(menu_builder, gtk::PopoverMenu, popover_menu);
        open_menu_btn.set_popover(Some(&popover_menu));

        get_widget!(self.builder, gtk::Stack, content);
        content.add_named(&self.previewer.widget, "previewer");

        // Recents Popover
        get_widget!(self.builder, gtk::MenuButton, recents_btn);
        let recents_popover = RecentsPopover::new(&history.model, self.sender.clone());
        recents_btn.set_popover(Some(&recents_popover.widget));

        // Export Popover
        get_widget!(self.builder, gtk::MenuButton, export_btn);
        self.exporter.widget.set_relative_to(Some(&export_btn));
        export_btn.set_popover(Some(&self.exporter.widget));

        self.widget.show_all();
    }

    fn setup_actions(&self) {
        // Export icon
        action!(
            self.widget,
            "export",
            clone!(@strong self.exporter as exporter => move |_, _| {
                exporter.widget.popup();
            })
        );

        action!(
            self.widget,
            "export-save",
            Some(&glib::VariantTy::new("s").unwrap()),
            clone!(@strong self.open_project as project, @weak self.widget as parent => move |_, target| {
                if let Some(ref project) = * project.borrow() {
                    let project_type = target.unwrap().get_str().unwrap();
                    if project.export(project_type, &parent.upcast::<gtk::Window>()).is_err() {
                        warn!("Failed to export the project");
                    }
                }
            })
        );

        // New Project
        action!(
            self.widget,
            "new-project",
            clone!(@weak self.widget as window, @strong self.sender as sender => move |_, _| {
                let dialog = NewProjectDialog::new(sender.clone());
                dialog.widget.set_transient_for(Some(&window));
                dialog.widget.show();
            })
        );

        // Refresh
        action!(
            self.widget,
            "refresh",
            clone!(@strong self.sender as sender, @strong self.open_project as project => move |_, _| {
                if let Some(ref project) = * project.borrow() {
                   match Project::parse(project.file.clone()) {
                        Ok(project) => send!(sender, Action::OpenProject(project)),
                        Err(err) => warn!("Failed to parse the project {}", err),
                    }
                }
            })
        );

        // Shuffle sample icons
        action!(
            self.widget,
            "shuffle",
            clone!(@strong self.previewer as previewer => move |_, _| {
                previewer.shuffle_samples();
            })
        );

        // Screenshot
        action!(
            self.widget,
            "screenshot",
            clone!(@weak self.widget as window, @strong self.previewer as previewer => move |_, _| {
                if let Some(pixbuf) = previewer.screenshot() {
                    let dialog = ScreenshotDialog::new(pixbuf);
                    dialog.widget.set_transient_for(Some(&window));
                    dialog.widget.show_all();
                }
            })
        );

        // Screenshot
        action!(
            self.widget,
            "copy-screenshot",
            clone!(@strong self.previewer as previewer => move |_, _| {
                if let Some(pixbuf) = previewer.screenshot() {
                    let dialog = ScreenshotDialog::new(pixbuf);
                    dialog.copy();
                }
            })
        );

        // About
        action!(
            self.widget,
            "about",
            clone!(@weak self.widget as window => move |_, _| {
                let builder = gtk::Builder::new_from_resource("/org/gnome/design/AppIconPreview/about_dialog.ui");
                get_widget!(builder, gtk::AboutDialog, about_dialog);
                about_dialog.set_transient_for(Some(&window));

                about_dialog.connect_response(|dialog, _| dialog.destroy());
                about_dialog.show();

            })
        );
        // Open file
        action!(
            self.widget,
            "open",
            clone!(@weak self.widget as window, @strong self.sender as sender => move |_, _| {
                let file_chooser = gtk::FileChooserNative::new(Some(&gettext("Open File")),
                                        Some(&window), gtk::FileChooserAction::Open,
                                        None, None);

                let svg_filter = gtk::FileFilter::new();
                svg_filter.set_name(Some(&gettext("SVG images")));
                svg_filter.add_mime_type("image/svg+xml");

                file_chooser.add_filter(&svg_filter);

                if file_chooser.run() == gtk::ResponseType::Accept {
                    if let Some(file) = file_chooser.get_file() {
                        match Project::parse(file) {
                            Ok(project) => send!(sender, Action::OpenProject(project)),
                            Err(err) => warn!("Failed to open file {}", err),
                        }
                    }
                }
                file_chooser.destroy();
            })
        );
    }

    fn init(&self) {
        // Devel Profile
        if PROFILE == "Devel" {
            self.widget.get_style_context().add_class("devel");
        }

        // load latest window state
        let width = SettingsManager::get_integer(Key::WindowWidth);
        let height = SettingsManager::get_integer(Key::WindowHeight);
        if width > -1 && height > -1 {
            self.widget.resize(width, height);
        }
        let x = SettingsManager::get_integer(Key::WindowX);
        let y = SettingsManager::get_integer(Key::WindowY);
        let is_maximized = SettingsManager::get_boolean(Key::IsMaximized);

        if x > -1 && y > -1 {
            self.widget.move_(x, y);
        } else if is_maximized {
            self.widget.maximize();
        }

        // save window state on delete event
        self.widget.connect_delete_event(move |window, _| {
            let size = window.get_size();
            let position = window.get_position();

            SettingsManager::set_integer(Key::WindowWidth, size.0);
            SettingsManager::set_integer(Key::WindowHeight, size.1);
            SettingsManager::set_boolean(Key::IsMaximized, window.is_maximized());
            SettingsManager::set_integer(Key::WindowX, position.0);
            SettingsManager::set_integer(Key::WindowY, position.1);

            Inhibit(false)
        });
    }
}
