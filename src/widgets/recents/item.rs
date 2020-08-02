use gtk::prelude::*;

use crate::project::Project;
use std::rc::Rc;

#[derive(Clone, Serialize, Deserialize)]
pub struct RecentItem {
    pub uri: String,
}

pub struct RecentItemRow {
    pub widget: gtk::FlowBoxChild,
    pub event_box: gtk::EventBox,
    project: Rc<Project>,
}

impl RecentItemRow {
    pub fn new(project: Rc<Project>) -> Self {
        let widget = gtk::FlowBoxChild::new();
        let event_box = gtk::EventBox::new();

        let recent_item = Self { widget, project, event_box };
        recent_item.init();
        recent_item
    }

    fn init(&self) {
        let container = gtk::Box::new(gtk::Orientation::Horizontal, 12);
        container.set_border_width(6);
        container.set_property_height_request(50);

        if let Ok((hicolor, _)) = self.project.get_hicolor(None) {
            let image = gtk::Image::new_from_gicon(&gio::FileIcon::new(&hicolor), gtk::IconSize::Dnd);
            image.get_style_context().add_class("icon-dropshadow");
            container.pack_start(&image, false, false, 0);
        }
        let project_name = self.project.name();

        let item_label = gtk::Label::new(Some(&project_name));
        item_label.set_halign(gtk::Align::Start);
        item_label.set_valign(gtk::Align::Center);
        item_label.set_ellipsize(pango::EllipsizeMode::End);
        item_label.set_tooltip_text(Some(&project_name));
        item_label.get_style_context().add_class("recent-item");
        container.pack_start(&item_label, true, true, 0);

        self.event_box.add(&container);
        self.widget.add(&self.event_box);
        self.widget.show_all();
    }
}
