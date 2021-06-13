use crate::common;

use gtk::prelude::*;
use gtk::{gio, pango};

#[derive(Clone)]
pub struct Icon {
    pub widget: gtk::Box,
    image: gtk::Image,
    pub label: gtk::Label,
    file: Option<gio::File>,
    size: i32,
}

impl Icon {
    pub fn new(file: Option<gio::File>, size: i32) -> Self {
        let widget = gtk::Box::new(gtk::Orientation::Vertical, 6);
        let image = gtk::Image::new();
        let label = gtk::Label::new(None);

        let icon = Self { widget, image, file, label, size };
        icon.init();
        icon
    }

    pub fn set_file(&self, file: &gio::File) {
        let filename = file.basename().unwrap();

        self.label.set_text(&common::format_name(filename.to_str().unwrap()));

        let gicon = gio::FileIcon::new(file);
        self.image.set_from_gicon(&gicon);
    }

    fn init(&self) {
        self.widget.set_valign(gtk::Align::Center);
        self.widget.set_margin_start(15);
        self.widget.set_margin_end(15);
        self.widget.set_margin_top(15);
        self.widget.set_margin_bottom(15);

        self.image.set_pixel_size(self.size);
        self.image.add_css_class("icon-dropshadow");

        self.label.set_ellipsize(pango::EllipsizeMode::End);
        self.label.set_max_width_chars(30);

        self.widget.prepend(&self.image);
        self.widget.append(&self.label);
    }
}
