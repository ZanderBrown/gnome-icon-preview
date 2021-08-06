use super::common;
use crate::common::Icon;

use gettextrs::gettext;
use rsvg::{CairoRenderer, Loader, SvgHandle};
use std::rc::Rc;

use gtk::glib::clone;
use gtk::prelude::*;
use gtk::{gio, glib};

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

    pub fn parse(file: gio::File) -> anyhow::Result<Rc<Self>> {
        let stream = file.read(gio::NONE_CANCELLABLE)?.upcast::<gio::InputStream>();
        let mut handle = Loader::new().read_stream(&stream, Some(&file), gio::NONE_CANCELLABLE)?;
        handle.set_stylesheet("#layer3,#layer2 {opacity: 0}")?;
        let renderer = CairoRenderer::new(&handle);

        let dimensions = renderer.intrinsic_dimensions();

        let width = dimensions.width.unwrap().length;
        let height = dimensions.height.unwrap().length;

        if (width - 128.0).abs() < std::f64::EPSILON && (height - 128.0).abs() < std::f64::EPSILON {
            return Ok(Rc::new(Self {
                project_type: ProjectType::Preview,
                file,
                handle,
            }));
        }

        if handle.has_element_with_id("#hicolor")? && handle.has_element_with_id("#symbolic")? {
            return Ok(Rc::new(Self {
                file,
                project_type: ProjectType::Icon,
                handle,
            }));
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

    pub fn export(&self, icon_type: &str, parent: &gtk::Window) -> anyhow::Result<()> {
        let mut icon_name: String = self.name();
        let mut gicon: Option<gio::File> = None;
        match icon_type {
            "nightly" => {
                icon_name = format!("{}.Devel.svg", self.name());
                gicon = Some(self.get_nightly()?);
            }
            "regular" => {
                icon_name = format!("{}.svg", self.name());
                gicon = Some(self.get_hicolor(None)?.0);
            }
            "symbolic" => {
                icon_name = format!("{}-symbolic.svg", self.name());
                gicon = Some(self.get_symbolic()?.0);
            }
            _ => (),
        };

        let dialog = gtk::FileChooserNative::new(Some(&gettext("Export")), Some(parent), gtk::FileChooserAction::Save, Some(&gettext("_Save")), Some(&gettext("_Cancel")));
        dialog.set_modal(true);
        dialog.set_current_name(&icon_name);

        let svg_filter = gtk::FileFilter::new();
        svg_filter.set_name(Some(&gettext("SVG")));
        svg_filter.add_pattern("*.svg");
        svg_filter.add_mime_type("image/svg+xml");
        dialog.add_filter(&svg_filter);

        dialog.connect_response(clone!(@strong gicon, @strong dialog => move |_, response| {
            if response == gtk::ResponseType::Accept {
                let dest = dialog.file().unwrap();
                if let Some(source) = &gicon {
                    let save = move ||  -> anyhow::Result<()> {
                        let (svg, _) = source.load_contents(gio::NONE_CANCELLABLE)?;
                        let cleaned_svg = common::clean_svg(std::str::from_utf8(&svg)?)?;

                        dest.replace_contents(&cleaned_svg, None, false,
                                                gio::FileCreateFlags::REPLACE_DESTINATION,
                                                gio::NONE_CANCELLABLE)?;
                        Ok(())
                    };
                    if save().is_err() {
                        log::warn!("Failed to save/clean the SVG file");
                    }
                }
            }
            dialog.destroy();
        }));
        dialog.show();
        Ok(())
    }

    pub fn get_hicolor(&self, _dest: Option<std::path::PathBuf>) -> anyhow::Result<(gio::File, cairo::SvgSurface)> {
        match self.project_type {
            ProjectType::Icon => common::render_by_id(&self.handle, &self.name(), Icon::Scalable, "#hicolor", 128.0),
            ProjectType::Preview => common::render(&self.handle, &self.name(), Icon::Scalable, 128.0),
        }
    }

    pub fn get_symbolic(&self) -> anyhow::Result<(gio::File, cairo::SvgSurface)> {
        match self.project_type {
            ProjectType::Icon => common::render_by_id(&self.handle, &self.name(), Icon::Symbolic, "#symbolic", 16.0),
            ProjectType::Preview => anyhow::bail!("No symbolic support for Preview icons"),
        }
    }

    pub fn get_nightly(&self) -> anyhow::Result<gio::File> {
        let dest_path = common::create_tmp(Icon::Devel, &self.name())?;
        let dest = gio::File::for_path(&dest_path);

        let source = match self.project_type {
            ProjectType::Icon => common::render_by_id(&self.handle, &self.name(), Icon::Devel, "#hicolor", 128.0),
            ProjectType::Preview => common::render(&self.handle, &self.name(), Icon::Devel, 128.0),
        }?
        .1;
        common::render_stripes(&source, 128.0)?;
        Ok(dest)
    }
}

#[cfg(test)]
mod tests {
    use super::{Project, ProjectType};
    use gtk::gio;
    #[test]
    fn parsing() {
        let project = Project::parse(gio::File::for_path("./tests/com.belmoussaoui.ReadItLater.Source.svg")).unwrap();
        assert_eq!(project.project_type, ProjectType::Icon);
        assert_eq!(project.get_symbolic().is_err(), false);
        assert_eq!(project.get_hicolor(None).is_err(), false);

        let project = Project::parse(gio::File::for_path("./tests/org.gnome.Test.Source.svg")).unwrap();
        assert_eq!(project.project_type, ProjectType::Icon);
        assert_eq!(project.get_symbolic().is_err(), false);
        assert_eq!(project.get_hicolor(None).is_err(), false);

        let project = Project::parse(gio::File::for_path("./tests/org.gnome.design.BannerViewer.svg")).unwrap();
        assert_eq!(project.project_type, ProjectType::Preview);
        assert_ne!(project.get_symbolic().is_err(), false);
        assert_eq!(project.get_hicolor(None).is_err(), false);
    }
}
