use super::common;
use crate::common::Icon;

use gettextrs::gettext;
use rsvg::{CairoRenderer, Loader, SvgHandle};
use std::rc::Rc;

use gtk::prelude::*;
use gtk::{gdk, gio, glib};

#[derive(PartialEq, Debug, Clone)]
pub enum ProjectType {
    Icon,    // A #hicolor & #symbolic found
    Preview, // A 128px SVG found
}

pub struct Project {
    pub file: gio::File,
    pub project_type: ProjectType,
    handle: SvgHandle,
}

impl Project {
    pub fn from_template(dest: gio::File) -> anyhow::Result<Rc<Self>> {
        let template = gio::File::for_uri("resource://org/gnome/design/AppIconPreview/templates/empty_project.svg");
        // Creates the parent directory tree if it does not already exist
        dest.parent().map(|parent| parent.make_directory_with_parents(gio::NONE_CANCELLABLE));

        template.copy(&dest, gio::FileCopyFlags::OVERWRITE, gio::NONE_CANCELLABLE, None)?;
        Project::parse(dest)
    }

    pub fn cache_icons(&self) -> anyhow::Result<()> {
        match self.project_type {
            ProjectType::Icon => {
                common::render_by_id(&self.handle, &self.name(), Icon::Scalable)?;
                common::render_by_id(&self.handle, &self.name(), Icon::Devel)?;
                common::render_by_id(&self.handle, &self.name(), Icon::Symbolic)?;
            }
            ProjectType::Preview => {
                common::render(&self.handle, &self.name(), Icon::Scalable)?;
                common::render(&self.handle, &self.name(), Icon::Devel)?;
            }
        }

        let app = gio::Application::default().unwrap().downcast::<crate::Application>().unwrap();

        // We need to refresh the search path after caching icons.
        app.icon_theme().add_search_path(glib::user_cache_dir().join("app-icon-preview").join("icons"));

        Ok(())
    }

    pub fn parse(file: gio::File) -> anyhow::Result<Rc<Self>> {
        let stream = file.read(gio::NONE_CANCELLABLE)?.upcast::<gio::InputStream>();
        let mut handle = Loader::new().read_stream(&stream, Some(&file), gio::NONE_CANCELLABLE)?;
        handle.set_stylesheet("#layer3,#layer2 {opacity: 0}")?;
        let renderer = CairoRenderer::new(&handle);

        let dimensions = renderer.intrinsic_dimensions();

        let width = dimensions.width.unwrap().length;
        let height = dimensions.height.unwrap().length;

        if (width - 128.0).abs() < std::f64::EPSILON && (height - 128.0).abs() < std::f64::EPSILON {
            let project = Self {
                project_type: ProjectType::Preview,
                file,
                handle,
            };
            project.cache_icons()?;
            return Ok(Rc::new(project));
        }

        if handle.has_element_with_id("#hicolor")? && handle.has_element_with_id("#symbolic")? {
            let project = Self {
                file,
                project_type: ProjectType::Icon,
                handle,
            };
            project.cache_icons()?;
            return Ok(Rc::new(project));
        }

        anyhow::bail!("not found")
    }

    pub fn name(&self) -> String {
        let filename = self.file.basename().unwrap();
        let filename = filename.to_str().unwrap().trim_end_matches(".svg").trim_end_matches(".Source");
        filename.to_string()
    }

    pub fn uri(&self) -> String {
        self.file.uri().to_string()
    }

    #[allow(dead_code)]
    pub fn open(&self) {
        let uri = self.file.uri();
        glib::idle_add(move || {
            if let Err(err) = gio::AppInfo::launch_default_for_uri(&uri, None::<&gio::AppLaunchContext>) {
                log::error!("Failed to open the project in Inkscape {}", err);
            }
            glib::Continue(false)
        });
    }

    pub async fn export(&self, icon_type: &str, parent: &gtk::Window) -> anyhow::Result<()> {
        let (basename, icon) = match icon_type {
            "nightly" => (format!("{}.Devel.svg", self.name()), Icon::Devel),
            "regular" => (format!("{}.svg", self.name()), Icon::Scalable),
            "symbolic" => (format!("{}-symbolic.svg", self.name()), Icon::Symbolic),
            _ => unimplemented!(),
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
            let (bytes, _) = gfile.load_contents_async_future().await?;
            let cleaned_svg = common::clean_svg(std::str::from_utf8(&bytes)?)?;

            if let Err(err) = dest.replace_contents_async_future(cleaned_svg, None, false, gio::FileCreateFlags::REPLACE_DESTINATION).await {
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

        has_scalable && has_devel && (has_symbolic || self.project_type == ProjectType::Preview)
    }
}
