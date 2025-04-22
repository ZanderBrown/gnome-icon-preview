use gtk::{glib, pango, prelude::*, subclass::prelude::*};

use crate::project::Project;

mod imp {
    use std::cell::OnceCell;

    use super::*;

    #[derive(Debug, Default, glib::Properties)]
    #[properties(wrapper_type = super::RecentItemRow)]
    pub struct RecentItemRow {
        #[property(get, set = Self::set_project, construct_only)]
        project: OnceCell<Project>,
        pub image: gtk::Image,
        pub label: gtk::Label,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for RecentItemRow {
        const NAME: &'static str = "RecentItemRow";
        type Type = super::RecentItemRow;
        type ParentType = gtk::FlowBoxChild;
    }

    #[glib::derived_properties]
    impl ObjectImpl for RecentItemRow {
        fn constructed(&self) {
            self.parent_constructed();

            let container = gtk::Box::new(gtk::Orientation::Horizontal, 12);

            self.image.set_icon_size(gtk::IconSize::Large);
            self.image.add_css_class("icon-dropshadow");
            container.append(&self.image);

            self.label.set_xalign(0.0);
            self.label.set_ellipsize(pango::EllipsizeMode::End);
            self.label.add_css_class("recent-item");
            container.append(&self.label);

            self.obj().set_child(Some(&container));
        }
    }
    impl WidgetImpl for RecentItemRow {}
    impl FlowBoxChildImpl for RecentItemRow {}

    impl RecentItemRow {
        fn set_project(&self, project: Project) {
            let project_name = project.name();

            if !project.has_cache_icons() {
                if let Err(err) = project.cache_icons() {
                    log::error!("Failed to cache icons for {}: {}", project_name, err);
                }
            }
            let scale = self.obj().scale_factor();
            let paintable = project.paintable(crate::common::Icon::Scalable, None, scale);
            self.image.set_paintable(paintable.as_ref());
            self.label.set_label(&project_name);
            self.label.set_tooltip_text(Some(&project_name));
            self.project.set(project).unwrap();
        }
    }
}

glib::wrapper! {
    pub struct RecentItemRow(ObjectSubclass<imp::RecentItemRow>)
        @extends gtk::Widget, gtk::FlowBoxChild;
}

impl RecentItemRow {
    pub fn new(project: Project) -> Self {
        glib::Object::builder()
            .property("project", &project)
            .build()
    }
}
