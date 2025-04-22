use gtk::{glib, prelude::*, subclass::prelude::*};

use crate::project::{Project, ProjectType};

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
            klass.bind_template();
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
    pub fn set_project(&self, project: &Project) {
        use crate::common::Icon;
        let imp = self.imp();

        // We get the scale_factor from the parent widget. For some reason the
        // popover will always report 1.
        let scale = self.parent().map_or(1, |p| p.scale_factor());
        imp.regular_image
            .set_paintable(project.paintable(Icon::Scalable, None, scale).as_ref());
        imp.nightly_image
            .set_paintable(project.paintable(Icon::Devel, None, scale).as_ref());

        let has_symbolic = project.project_type() == ProjectType::Icon;
        imp.symbolic_box.set_visible(has_symbolic);
        if has_symbolic {
            imp.symbolic_image
                .set_paintable(project.paintable(Icon::Symbolic, None, scale).as_ref());
        }
    }
}

impl Default for ExportPopover {
    fn default() -> Self {
        glib::Object::new()
    }
}
