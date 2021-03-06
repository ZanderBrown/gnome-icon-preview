use gtk::prelude::*;

use crate::project::Project;

#[derive(Clone)]
pub struct ExportPopover {
    pub widget: gtk::Popover,
    builder: gtk::Builder,
}

impl ExportPopover {
    pub fn new() -> Self {
        let builder = gtk::Builder::new_from_resource("/org/gnome/design/AppIconPreview/export.ui");
        get_widget!(builder, gtk::Popover, export_popover);
        Self { builder, widget: export_popover }
    }

    pub fn set_project(&self, project: &Project) {
        match project.get_hicolor(None) {
            Ok((hicolor, _)) => {
                get_widget!(self.builder, gtk::Image, regular_image);
                regular_image.set_from_gicon(&gio::FileIcon::new(&hicolor), gtk::IconSize::Dialog);
                get_widget!(self.builder, gtk::Box, @regular_box).show();
            }
            Err(_) => get_widget!(self.builder, gtk::Box, @regular_box).hide(),
        };

        match project.get_symbolic() {
            Ok((symbolic, _)) => {
                get_widget!(self.builder, gtk::Image, symbolic_image);
                symbolic_image.set_from_gicon(&gio::FileIcon::new(&symbolic), gtk::IconSize::Dialog);
                get_widget!(self.builder, gtk::Box, @symbolic_box).show();
            }
            Err(_) => get_widget!(self.builder, gtk::Box, @symbolic_box).hide(),
        };

        match project.get_nightly() {
            Ok(nightly) => {
                get_widget!(self.builder, gtk::Image, nightly_image);
                nightly_image.set_from_gicon(&gio::FileIcon::new(&nightly), gtk::IconSize::Dialog);
                get_widget!(self.builder, gtk::Box, @nightly_box).show();
            }
            Err(_) => get_widget!(self.builder, gtk::Box, @nightly_box).hide(),
        };
    }
}
