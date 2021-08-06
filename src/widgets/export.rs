use crate::project::Project;

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
        match project.get_hicolor(None) {
            Ok(_) => {
                get_widget!(self.builder, gtk::Image, regular_image);
                regular_image.set_icon_name(Some(&project.name()));
                get_widget!(self.builder, gtk::Box, @regular_box).show();
            }
            Err(_) => get_widget!(self.builder, gtk::Box, @regular_box).hide(),
        };

        match project.get_symbolic() {
            Ok(_) => {
                get_widget!(self.builder, gtk::Image, symbolic_image);
                symbolic_image.set_icon_name(Some(&format!("{}-symbolic", project.name())));
                get_widget!(self.builder, gtk::Box, @symbolic_box).show();
            }
            Err(_) => get_widget!(self.builder, gtk::Box, @symbolic_box).hide(),
        };

        match project.get_nightly() {
            Ok(_) => {
                get_widget!(self.builder, gtk::Image, nightly_image);
                nightly_image.set_icon_name(Some(&format!("{}.Devel", project.name())));
                get_widget!(self.builder, gtk::Box, @nightly_box).show();
            }
            Err(_) => get_widget!(self.builder, gtk::Box, @nightly_box).hide(),
        };
    }
}
