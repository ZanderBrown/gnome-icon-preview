use crate::project::Project;

use serde_derive::{Deserialize, Serialize};

use gtk::prelude::*;
use gtk::{glib, pango};

#[derive(Clone, Serialize, Deserialize)]
pub struct RecentItem {
    pub uri: String,
}
use gtk::subclass::prelude::*;

mod imp {
    use super::*;
    use glib::{ParamFlags, ParamSpec, Value};
    use once_cell::sync::{Lazy, OnceCell};

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

        fn new() -> Self {
            Self {
                project: Default::default(),
                image: gtk::Image::new(),
                label: gtk::Label::new(None),
            }
        }
    }

    impl ObjectImpl for RecentItemRow {
        fn constructed(&self, obj: &Self::Type) {
            let container = gtk::Box::new(gtk::Orientation::Horizontal, 12);

            self.image.set_icon_size(gtk::IconSize::Large);
            self.image.add_css_class("icon-dropshadow");
            container.append(&self.image);

            self.label.set_xalign(0.0);
            self.label.set_ellipsize(pango::EllipsizeMode::End);
            self.label.add_css_class("recent-item");
            container.append(&self.label);

            obj.set_child(Some(&container));
            self.parent_constructed(obj);
        }

        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![ParamSpec::new_object(
                    "project",
                    "Project",
                    "The associated recent project",
                    Project::static_type(),
                    ParamFlags::READWRITE | ParamFlags::CONSTRUCT,
                )]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, obj: &Self::Type, _id: usize, value: &Value, pspec: &ParamSpec) {
            match pspec.name() {
                "project" => {
                    let project = value.get().unwrap();
                    obj.set_project(&project);
                    self.project.set(project).unwrap();
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> Value {
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
        glib::Object::new(&[("project", &project)]).unwrap()
    }

    fn set_project(&self, project: &Project) {
        let self_ = imp::RecentItemRow::from_instance(self);
        let project_name = project.name();

        if !project.has_cache_icons() {
            if let Err(err) = project.cache_icons() {
                log::error!("Failed to cache icons for {}: {}", project_name, err);
            }
        }

        self_.image.set_icon_name(Some(&project_name));
        self_.label.set_label(&project_name);
        self_.label.set_tooltip_text(Some(&project_name));
    }
}
