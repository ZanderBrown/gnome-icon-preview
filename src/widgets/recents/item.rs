use crate::project::Project;

use std::rc::Rc;

use gtk::prelude::*;
use gtk::{gio, pango};

#[derive(Clone, Serialize, Deserialize)]
pub struct RecentItem {
    pub uri: String,
}

pub struct RecentItemRow {
    pub widget: gtk::FlowBoxChild,
    project: Rc<Project>,
}

impl RecentItemRow {
    pub fn new(project: Rc<Project>) -> Self {
        let widget = gtk::FlowBoxChild::new();

        let recent_item = Self { widget, project };
        recent_item.init();
        recent_item
    }

    fn init(&self) {
        let container = gtk::Box::new(gtk::Orientation::Horizontal, 12);
        container.set_height_request(50);
        container.set_margin_start(12);
        container.set_margin_end(12);

        if let Ok((hicolor, _)) = self.project.get_hicolor(None) {
            let image = gtk::Image::from_gicon(&gio::FileIcon::new(&hicolor));
            image.set_icon_size(gtk::IconSize::Large);
            image.add_css_class("icon-dropshadow");
            container.append(&image);
        }
        let project_name = self.project.name();

        let item_label = gtk::Label::new(Some(&project_name));
        item_label.set_halign(gtk::Align::Start);
        item_label.set_valign(gtk::Align::Center);
        item_label.set_ellipsize(pango::EllipsizeMode::End);
        item_label.set_tooltip_text(Some(&project_name));
        item_label.add_css_class("recent-item");
        container.append(&item_label);

        self.widget.set_child(Some(&container));
    }
}
