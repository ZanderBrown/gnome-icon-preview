use crate::project::{Project, ProjectType};

use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(resource = "/org/gnome/design/AppIconPreview/export.ui")]
    pub struct ExportPopover {
        #[template_child]
        pub regular_image: TemplateChild<gtk::Image>,
        #[template_child]
        pub symbolic_image: TemplateChild<gtk::Image>,
        #[template_child]
        pub nightly_image: TemplateChild<gtk::Image>,
        #[template_child]
        pub symbolic_box: TemplateChild<gtk::Box>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ExportPopover {
        const NAME: &'static str = "ExportPopover";
        type Type = super::ExportPopover;
        type ParentType = gtk::Popover;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ExportPopover {}
    impl WidgetImpl for ExportPopover {}
    impl PopoverImpl for ExportPopover {}
}

glib::wrapper! {
    pub struct ExportPopover(ObjectSubclass<imp::ExportPopover>)
        @extends gtk::Widget, gtk::Popover;
}

impl ExportPopover {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        glib::Object::new(&[]).unwrap()
    }

    pub fn set_project(&self, project: &Project) {
        let self_ = imp::ExportPopover::from_instance(self);

        self_.regular_image.set_icon_name(Some(&project.name()));
        self_.nightly_image.set_icon_name(Some(&format!("{}.Devel", project.name())));

        let has_symbolic = project.project_type() == ProjectType::Icon;
        self_.symbolic_box.set_visible(has_symbolic);
        if has_symbolic {
            self_.symbolic_image.set_icon_name(Some(&format!("{}-symbolic", project.name())));
        }
    }
}
