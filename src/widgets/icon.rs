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

    #[derive(Debug, Default)]
    pub struct Icon {
        pub image: gtk::Image,
        pub label: gtk::Label,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Icon {
        const NAME: &'static str = "AppIcon";
        type Type = super::Icon;
        type ParentType = gtk::Box;
    }

    impl ObjectImpl for Icon {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();

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
        let icon = glib::Object::builder::<Self>().property("orientation", gtk::Orientation::Vertical).property("spacing", 6).build();

        let imp = icon.imp();
        match size {
            IconSize::Small => {
                imp.label.add_css_class("caption");
                imp.image.set_pixel_size(64);
            }
            IconSize::Large => {
                imp.image.set_pixel_size(96);
            }
        };
        icon
    }

    pub fn set_file(&self, file: &gio::File) {
        let imp = self.imp();
        let filename = file.basename().unwrap();

        imp.label.set_text(&common::format_name(filename.to_str().unwrap()));

        let gicon = gio::FileIcon::new(file);
        imp.image.set_from_gicon(&gicon);
    }

    pub fn set_icon_name(&self, icon_name: &str) {
        let imp = self.imp();

        imp.label.set_text(&common::format_name(icon_name));
        imp.image.set_icon_name(Some(icon_name));
    }
}
