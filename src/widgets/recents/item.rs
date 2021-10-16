use crate::project::Project;

use serde_derive::{Deserialize, Serialize};

use gtk::pango;
use gtk::prelude::*;

#[derive(Clone, Serialize, Deserialize)]
pub struct RecentItem {
    pub uri: String,
}

pub struct RecentItemRow {
    pub widget: gtk::FlowBoxChild,
    project: Project,
}

impl RecentItemRow {
    pub fn new(project: Project) -> Self {
        let widget = gtk::FlowBoxChild::new();

        let recent_item = Self { widget, project };
        recent_item.init();
        recent_item
    }

    fn init(&self) {
        let container = gtk::Box::new(gtk::Orientation::Horizontal, 12);

        let project_name = self.project.name();

        if !self.project.has_cache_icons() {
            if let Err(err) = self.project.cache_icons() {
                log::error!("Failed to cache icons for {}: {}", self.project.name(), err);
            }
        }

        let image = gtk::Image::from_icon_name(Some(&project_name));
        image.set_icon_size(gtk::IconSize::Large);
        image.add_css_class("icon-dropshadow");
        container.append(&image);

        let item_label = gtk::Label::new(Some(&project_name));
        item_label.set_xalign(0.0);
        item_label.set_ellipsize(pango::EllipsizeMode::End);
        item_label.set_tooltip_text(Some(&project_name));
        item_label.add_css_class("recent-item");
        container.append(&item_label);

        self.widget.set_child(Some(&container));
    }
}
