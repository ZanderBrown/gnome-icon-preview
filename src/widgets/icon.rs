use crate::common;

use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gio, glib, pango};

pub enum IconSize {
    Small,
    Large,
}

mod imp {
    use super::*;
    use std::cell::Cell;

    #[derive(Debug)]
    pub struct Icon {
        pub image: gtk::Image,
        pub label: gtk::Label,
        pub size: Cell<i32>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Icon {
        const NAME: &'static str = "AppIcon";
        type Type = super::Icon;
        type ParentType = gtk::Box;

        fn new() -> Self {
            Self {
                image: gtk::Image::new(),
                label: gtk::Label::new(None),
                size: Cell::new(-1),
            }
        }
    }

    impl ObjectImpl for Icon {
        fn constructed(&self, obj: &Self::Type) {
            obj.set_valign(gtk::Align::Center);
            obj.set_margin_start(15);
            obj.set_margin_end(15);
            obj.set_margin_top(15);
            obj.set_margin_bottom(15);

            self.image.add_css_class("icon-dropshadow");

            self.label.set_ellipsize(pango::EllipsizeMode::End);
            self.label.set_max_width_chars(30);

            obj.prepend(&self.image);
            obj.append(&self.label);
            self.parent_constructed(obj);
        }
    }
    impl WidgetImpl for Icon {}
    impl BoxImpl for Icon {}
}

glib::wrapper! {
    pub struct Icon(ObjectSubclass<imp::Icon>)
        @extends gtk::Widget, gtk::Box;
}

impl Icon {
    pub fn new(size: IconSize) -> Self {
        let icon = glib::Object::new::<Self>(&[("orientation", &gtk::Orientation::Vertical), ("spacing", &6)]).unwrap();

        let self_ = imp::Icon::from_instance(&icon);
        match size {
            IconSize::Small => {
                self_.label.add_css_class("caption");
                self_.image.set_pixel_size(64);
            }
            IconSize::Large => {
                self_.image.set_pixel_size(96);
            }
        };
        icon
    }

    pub fn set_file(&self, file: &gio::File) {
        let self_ = imp::Icon::from_instance(self);
        let filename = file.basename().unwrap();

        self_.label.set_text(&common::format_name(filename.to_str().unwrap()));

        let gicon = gio::FileIcon::new(file);
        self_.image.set_from_gicon(&gicon);
    }

    pub fn set_icon_name(&self, icon_name: &str) {
        let self_ = imp::Icon::from_instance(self);

        self_.label.set_text(&common::format_name(icon_name));
        self_.image.set_icon_name(Some(icon_name));
    }
}
