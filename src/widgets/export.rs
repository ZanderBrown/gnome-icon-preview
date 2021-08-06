use crate::project::{Project, ProjectType};

use gtk::prelude::*;
use gtk_macros::get_widget;

#[derive(Clone)]
pub struct ExportPopover {
    pub widget: gtk::Popover,
    builder: gtk::Builder,
}

impl ExportPopover {
    pub fn new() -> Self {
        let builder = gtk::Builder::from_resource("/org/gnome/design/AppIconPreview/export.ui");
        get_widget!(builder, gtk::Popover, export_popover);
        Self { builder, widget: export_popover }
    }

    pub fn set_project(&self, project: &Project) {
        get_widget!(self.builder, gtk::Image, regular_image);
        get_widget!(self.builder, gtk::Image, symbolic_image);
        get_widget!(self.builder, gtk::Image, nightly_image);

        get_widget!(self.builder, gtk::Box, symbolic_box);

        regular_image.set_icon_name(Some(&project.name()));
        nightly_image.set_icon_name(Some(&format!("{}.Devel", project.name())));

        let has_symbolic = project.project_type == ProjectType::Icon;
        symbolic_box.set_visible(has_symbolic);
        if has_symbolic {
            symbolic_image.set_icon_name(Some(&format!("{}-symbolic", project.name())));
        }
    }
}
