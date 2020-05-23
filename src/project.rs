use super::common;
use anyhow::anyhow;
use gettextrs::gettext;
use gio::prelude::FileExt;
use gtk::prelude::*;
use librsvg::{CairoRenderer, Loader};
use std::path::PathBuf;

#[derive(PartialEq, Debug, Clone)]
pub enum ProjectType {
    Icon,    // A #hicolor & #symbolic found
    Preview, // A 128px SVG found
}

#[derive(Debug, Clone)]
pub struct Project {
    pub file: gio::File,
    pub project_type: ProjectType,
}

impl Project {
    pub fn from_template(dest: gio::File) -> anyhow::Result<Self> {
        let template = gio::File::new_for_uri("resource://org/gnome/design/AppIconPreview/templates/empty_project.svg");
        template.copy(&dest, gio::FileCopyFlags::OVERWRITE, gio::NONE_CANCELLABLE, None)?;

        Ok(Project::parse(dest)?)
    }

    pub fn parse(file: gio::File) -> anyhow::Result<Self> {
        let path = file.get_path().ok_or_else(|| anyhow!("Failed to get the path"))?;

        let handle = Loader::new().read_path(&path)?;
        let renderer = CairoRenderer::new(&handle);
        let dimensions = renderer.intrinsic_dimensions();
        let width = dimensions.width.unwrap().length;
        let height = dimensions.height.unwrap().length;

        if (width - 128.0).abs() < std::f64::EPSILON && (height - 128.0).abs() < std::f64::EPSILON {
            return Ok(Self {
                project_type: ProjectType::Preview,
                file,
            });
        }

        if handle.has_element_with_id("#hicolor")? && handle.has_element_with_id("#symbolic")? {
            return Ok(Self {
                file,
                project_type: ProjectType::Icon,
            });
        }
        anyhow::bail!("not found")
    }

    pub fn name(&self) -> String {
        let filename = self.file.get_basename().unwrap();
        let filename = filename.to_str().unwrap().trim_end_matches(".svg").trim_end_matches(".Source");
        filename.to_string()
    }

    pub fn path(&self) -> PathBuf {
        self.file.get_path().unwrap()
    }

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

        let dialog = gtk::FileChooserDialog::with_buttons(
            Some(&gettext("Export")),
            Some(parent),
            gtk::FileChooserAction::Save,
            &[(&gettext("_Save"), gtk::ResponseType::Accept), (&gettext("_Cancel"), gtk::ResponseType::Cancel)],
        );
        dialog.set_modal(true);
        dialog.set_current_name(&icon_name);

        let svg_filter = gtk::FileFilter::new();
        svg_filter.set_name(Some(&gettext("SVG")));
        svg_filter.add_pattern("*.svg");
        svg_filter.add_mime_type("image/svg+xml");
        dialog.add_filter(&svg_filter);

        dialog.connect_response(clone!(@strong gicon => move |dialog, response| {
            if response == gtk::ResponseType::Accept {
                let dest = dialog.get_file().unwrap();
                if let Some(source) = &gicon {
                    let save = move ||  -> anyhow::Result<()> {
                        source.copy(&dest, gio::FileCopyFlags::OVERWRITE, gio::NONE_CANCELLABLE, None)?;
                        common::clean_svg(dest.get_path().unwrap())?;
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
            ProjectType::Icon => common::render_by_id(&self.file, "#hicolor", 128.0, dest),
            ProjectType::Preview => common::render(&self.file, 128.0, dest),
        }
    }

    pub fn get_symbolic(&self) -> anyhow::Result<(gio::File, cairo::SvgSurface)> {
        match self.project_type {
            ProjectType::Icon => {
                let dest = common::create_tmp(&format!("#symblic-16-{}-symbolic.svg", self.name()))?;
                common::render_by_id(&self.file, "#symbolic", 16.0, Some(dest))
            }
            ProjectType::Preview => anyhow::bail!("No symbolic support for Preview icons"),
        }
    }

    pub fn get_nightly(&self) -> anyhow::Result<gio::File> {
        let dest_path = common::create_tmp(&format!("#nightly-{}-{}", 128.0, self.name()))?;
        let dest = gio::File::new_for_path(&dest_path.clone());

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
