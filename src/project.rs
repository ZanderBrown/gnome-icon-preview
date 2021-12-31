use super::common;
use crate::common::Icon;

use gettextrs::gettext;
use rsvg::{CairoRenderer, Loader, SvgHandle};

use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gdk, gio, glib};

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum ProjectType {
    Icon,    // A #hicolor & #symbolic found
    Preview, // A 128px SVG found
}

impl Default for ProjectType {
    fn default() -> Self {
        Self::Icon
    }
}

mod imp {
    use super::*;

    use once_cell::sync::OnceCell;
    use std::cell::Cell;

    #[derive(Default)]
    pub struct Project {
        pub file: OnceCell<gio::File>,
        pub project_type: Cell<ProjectType>,
        pub handle: OnceCell<SvgHandle>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Project {
        const NAME: &'static str = "Project";
        type Type = super::Project;
    }

    impl ObjectImpl for Project {}
}

glib::wrapper! {
    pub struct Project(ObjectSubclass<imp::Project>);
}

impl Default for Project {
    fn default() -> Self {
        glib::Object::new(&[]).unwrap()
    }
}

impl Project {
    pub fn from_template(dest: gio::File) -> anyhow::Result<Self> {
        let template = gio::File::for_uri("resource://org/gnome/design/AppIconPreview/templates/empty_project.svg");
        // Creates the parent directory tree if it does not already exist
        dest.parent().map(|parent| parent.make_directory_with_parents(gio::Cancellable::NONE));

        template.copy(&dest, gio::FileCopyFlags::OVERWRITE, gio::Cancellable::NONE, None)?;
        Project::parse(dest, true)
    }

    pub fn cache_icons(&self) -> anyhow::Result<()> {
        let name = self.name();
        let handle = self.imp().handle.get().unwrap();

        match self.project_type() {
            ProjectType::Icon => {
                common::render_by_id(handle, &name, Icon::Scalable).unwrap();
                common::render_by_id(handle, &name, Icon::Devel).unwrap();
                common::render_by_id(handle, &name, Icon::Symbolic).unwrap();
            }
            ProjectType::Preview => {
                common::render(handle, &name, Icon::Scalable).unwrap();
                common::render(handle, &name, Icon::Devel).unwrap();
            }
        }

        let app = gio::Application::default().unwrap().downcast::<crate::Application>().unwrap();

        // We need to refresh the search path after caching icons.
        app.icon_theme().add_search_path(common::icon_theme_path());

        Ok(())
    }

    pub fn parse(file: gio::File, cache_icons: bool) -> anyhow::Result<Self> {
        let stream = file.read(gio::Cancellable::NONE)?.upcast::<gio::InputStream>();
        let mut handle = Loader::new().read_stream(&stream, Some(&file), gio::Cancellable::NONE)?;
        handle.set_stylesheet("#layer3,#layer2 {opacity: 0}")?;
        let renderer = CairoRenderer::new(&handle);

        let dimensions = renderer.intrinsic_dimensions();

        let width = dimensions.width.map(|w| w.length).unwrap_or(-1.0);
        let height = dimensions.height.map(|h| h.length).unwrap_or(-1.0);

        if (width - Icon::Scalable.size()).abs() < std::f64::EPSILON && (height - Icon::Scalable.size()).abs() < std::f64::EPSILON {
            let project: Self = glib::Object::new(&[]).unwrap();
            let imp = project.imp();
            imp.project_type.set(ProjectType::Preview);
            imp.file.set(file).unwrap();
            let _ = imp.handle.set(handle);
            if cache_icons {
                project.cache_icons()?;
            }
            return Ok(project);
        }

        if handle.has_element_with_id(Icon::Scalable.id())? && handle.has_element_with_id(Icon::Symbolic.id())? {
            let project: Self = glib::Object::new(&[]).unwrap();
            let imp = project.imp();
            imp.project_type.set(ProjectType::Icon);
            imp.file.set(file).unwrap();
            let _ = imp.handle.set(handle);
            if cache_icons {
                project.cache_icons()?;
            }
            return Ok(project);
        }

        anyhow::bail!("not found")
    }

    pub fn name(&self) -> String {
        let filename = self.file().basename().unwrap();
        let filename = filename.to_str().unwrap().trim_end_matches(".svg").trim_end_matches(".Source");
        filename.to_string()
    }

    pub fn uri(&self) -> String {
        self.file().uri().to_string()
    }

    #[allow(dead_code)]
    pub fn open(&self) {
        let uri = self.file().uri();
        glib::idle_add(move || {
            if let Err(err) = gio::AppInfo::launch_default_for_uri(&uri, None::<&gio::AppLaunchContext>) {
                log::error!("Failed to open the project in Inkscape {}", err);
            }
            glib::Continue(false)
        });
    }

    pub async fn export(&self, icon: Icon, parent: &gtk::Window) -> anyhow::Result<()> {
        let basename = match icon {
            Icon::Devel => format!("{}.Devel.svg", self.name()),
            Icon::Scalable => format!("{}.svg", self.name()),
            Icon::Symbolic => format!("{}-symbolic.svg", self.name()),
        };

        let gfile = gio::File::for_path(icon.path().join(&basename));

        let dialog = gtk::FileChooserNative::new(Some(&gettext("Export")), Some(parent), gtk::FileChooserAction::Save, Some(&gettext("_Save")), Some(&gettext("_Cancel")));
        dialog.set_modal(true);
        dialog.set_current_name(&basename);

        let svg_filter = gtk::FileFilter::new();
        svg_filter.set_name(Some(&gettext("SVG")));
        svg_filter.add_pattern("*.svg");
        svg_filter.add_mime_type("image/svg+xml");
        dialog.add_filter(&svg_filter);

        if dialog.run_future().await == gtk::ResponseType::Accept {
            let dest = dialog.file().unwrap();
            let (bytes, _) = gfile.load_contents_future().await?;
            let cleaned_svg = common::clean_svg(std::str::from_utf8(&bytes)?)?;

            if let Err(err) = dest.replace_contents_future(cleaned_svg, None, false, gio::FileCreateFlags::REPLACE_DESTINATION).await {
                log::error!("Failed to export icon {:?}", err);
            };
        }

        Ok(())
    }

    pub fn has_cache_icons(&self) -> bool {
        let display = gdk::Display::default().unwrap();
        let icon_theme = gtk::IconTheme::for_display(&display).unwrap();

        let has_scalable = icon_theme.has_icon(&self.name());
        let has_devel = icon_theme.has_icon(&format!("{}.Devel", self.name()));
        let has_symbolic = icon_theme.has_icon(&format!("{}-symbolic", self.name()));

        has_scalable && has_devel && (has_symbolic || self.project_type() == ProjectType::Preview)
    }

    pub fn file(&self) -> gio::File {
        self.imp().file.get().unwrap().clone()
    }

    pub fn project_type(&self) -> ProjectType {
        self.imp().project_type.get()
    }
}
