use super::common;
use gettextrs::gettext;
use gio::prelude::FileExt;
use gtk::prelude::*;
use rsvg_internals::{Dpi, Handle, LoadOptions, SizeCallback};
use std::rc::Rc;

#[derive(PartialEq, Debug, Clone)]
pub enum ProjectType {
    Icon,    // A #hicolor & #symbolic found
    Preview, // A 128px SVG found
}

pub struct Project {
    pub file: gio::File,
    pub project_type: ProjectType,
    handle: Handle,
}

impl Project {
    pub fn from_template(dest: gio::File) -> anyhow::Result<Rc<Self>> {
        let template = gio::File::new_for_uri("resource://org/gnome/design/AppIconPreview/templates/empty_project.svg");
        // Creates the parent directory tree if it does not already exist
        dest.get_parent().map(|parent| parent.make_directory_with_parents(gio::NONE_CANCELLABLE));

        template.copy(&dest, gio::FileCopyFlags::OVERWRITE, gio::NONE_CANCELLABLE, None)?;
        Project::parse(dest)
    }

    pub fn parse(file: gio::File) -> anyhow::Result<Rc<Self>> {
        let stream = file.read(gio::NONE_CANCELLABLE)?.upcast::<gio::InputStream>();
        let mut handle = Handle::from_stream(&LoadOptions::new(None), &stream, gio::NONE_CANCELLABLE)?;
        handle.set_stylesheet("#layer3,#layer2 {visibility: hidden}")?;

        let dimensions = handle.get_dimensions(Dpi::default(), &SizeCallback::default(), false)?;

        let width = dimensions.width as f64;
        let height = dimensions.height as f64;

        if (width - 128.0).abs() < std::f64::EPSILON && (height - 128.0).abs() < std::f64::EPSILON {
            return Ok(Rc::new(Self {
                project_type: ProjectType::Preview,
                file,
                handle,
            }));
        }

        if handle.has_sub("#hicolor")? && handle.has_sub("#symbolic")? {
            return Ok(Rc::new(Self {
                file,
                project_type: ProjectType::Icon,
                handle,
            }));
        }
        anyhow::bail!("not found")
    }

    pub fn name(&self) -> String {
        let filename = self.file.get_basename().unwrap();
        let filename = filename.to_str().unwrap().trim_end_matches(".svg").trim_end_matches(".Source");
        filename.to_string()
    }

    pub fn uri(&self) -> String {
        self.file.get_uri().to_string()
    }

    #[allow(dead_code)]
    pub fn open(&self) {
        let uri = self.file.get_uri();
        gtk::idle_add(move || {
            if let Err(err) = gio::AppInfo::launch_default_for_uri(&uri, None::<&gio::AppLaunchContext>) {
                error!("Failed to open the project in Inkscape {}", err);
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
                let dest = dialog.get_file().unwrap();
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
                        warn!("Failed to save/clean the SVG file");
                    }
                }
            }
            dialog.destroy();
        }));
        dialog.show();
        Ok(())
    }

    pub fn get_hicolor(&self, dest: Option<std::path::PathBuf>) -> anyhow::Result<(gio::File, cairo::SvgSurface)> {
        match self.project_type {
            ProjectType::Icon => common::render_by_id(&self.handle, &self.name(), "#hicolor", 128.0, dest),
            ProjectType::Preview => common::render(&self.handle, &self.name(), 128.0, dest),
        }
    }

    pub fn get_symbolic(&self) -> anyhow::Result<(gio::File, cairo::SvgSurface)> {
        match self.project_type {
            ProjectType::Icon => {
                let dest = common::create_tmp(&format!("#symblic-16-{}-symbolic.svg", self.name()))?;
                common::render_by_id(&self.handle, &self.name(), "#symbolic", 16.0, Some(dest))
            }
            ProjectType::Preview => anyhow::bail!("No symbolic support for Preview icons"),
        }
    }

    pub fn get_nightly(&self) -> anyhow::Result<gio::File> {
        let dest_path = common::create_tmp(&format!("#nightly-{}-{}", 128.0, self.name()))?;
        let dest = gio::File::new_for_path(&dest_path);

        let (_, hicolor) = self.get_hicolor(Some(dest_path))?;

        common::render_stripes(&hicolor, 128.0)?;
        Ok(dest)
    }
}

#[cfg(test)]
mod tests {
    use super::{Project, ProjectType};
    #[test]
    fn parsing() {
        let project = Project::parse(gio::File::new_for_path("./tests/org.gnome.Test.Source.svg")).unwrap();
        assert_eq!(project.project_type, ProjectType::Icon);
        assert_eq!(project.get_symbolic().is_err(), false);
        assert_ne!(project.get_hicolor(None).is_err(), false);

        let project = Project::parse(gio::File::new_for_path("./tests/com.belmoussaoui.ReadItLater.Source.svg")).unwrap();
        assert_eq!(project.project_type, ProjectType::Icon);
        assert_eq!(project.get_symbolic().is_err(), false);
        assert_ne!(project.get_hicolor(None).is_err(), false);

        let project = Project::parse(gio::File::new_for_path("./tests/org.gnome.design.BannerViewer.svg")).unwrap();
        assert_eq!(project.project_type, ProjectType::Preview);
        assert_eq!(project.get_symbolic().is_err(), false);
        assert_ne!(project.get_hicolor(None).is_err(), false);
    }
}
