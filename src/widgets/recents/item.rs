use crate::project::Project;

use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{glib, pango};

mod imp {
    use super::*;
    use glib::{ParamSpec, ParamSpecObject, Value};
    use once_cell::sync::{Lazy, OnceCell};

    #[derive(Debug, Default)]
    pub struct RecentItemRow {
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

        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| vec![ParamSpecObject::builder::<Project>("project").construct().build()]);
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _id: usize, value: &Value, pspec: &ParamSpec) {
            match pspec.name() {
                "project" => {
                    let project = value.get().unwrap();
                    self.obj().set_project(&project);
                    self.project.set(project).unwrap();
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _id: usize, pspec: &ParamSpec) -> Value {
            match pspec.name() {
                "project" => self.project.get().to_value(),
                _ => unimplemented!(),
            }
        }
    }
    impl WidgetImpl for RecentItemRow {}
    impl FlowBoxChildImpl for RecentItemRow {}
}

glib::wrapper! {
    pub struct RecentItemRow(ObjectSubclass<imp::RecentItemRow>)
        @extends gtk::Widget, gtk::FlowBoxChild;
}

impl RecentItemRow {
    pub fn new(project: Project) -> Self {
        glib::Object::builder().property("project", &project).build()
    }

    fn set_project(&self, project: &Project) {
        let imp = self.imp();
        let project_name = project.name();

        if !project.has_cache_icons() {
            if let Err(err) = project.cache_icons() {
                log::error!("Failed to cache icons for {}: {}", project_name, err);
            }
        }

        imp.image.set_icon_name(Some(&project_name));
        imp.label.set_label(&project_name);
        imp.label.set_tooltip_text(Some(&project_name));
    }
}
