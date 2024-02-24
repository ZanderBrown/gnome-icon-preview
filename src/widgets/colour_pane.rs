use gtk::{gio, glib, prelude::*, subclass::prelude::*};

use super::icon::{Icon, IconSize};
use crate::project::Project;

#[derive(Debug, Default, Copy, Eq, PartialEq, Clone)]
pub enum PaneStyle {
    #[default]
    Light,
    Dark,
}

mod imp {
    use std::cell::{Cell, RefCell};

    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(resource = "/org/gnome/design/AppIconPreview/colourpane.ui")]
    pub struct ColourPane {
        pub style: Cell<PaneStyle>,
        pub small_icons: RefCell<Vec<Icon>>,
        pub grid_icons: RefCell<Vec<Icon>>,

        #[template_child]
        pub symbolic_image: TemplateChild<gtk::Image>,
        #[template_child]
        pub symbolic_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub hicolor_128: TemplateChild<gtk::Image>,
        #[template_child]
        pub hicolor_64: TemplateChild<gtk::Image>,
        #[template_child]
        pub hicolor_32: TemplateChild<gtk::Image>,
        #[template_child]
        pub grid: TemplateChild<gtk::Box>,
        #[template_child]
        pub small: TemplateChild<gtk::Box>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ColourPane {
        const NAME: &'static str = "ColourPane";
        type Type = super::ColourPane;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ColourPane {}
    impl WidgetImpl for ColourPane {}
    impl BoxImpl for ColourPane {}
}

glib::wrapper! {
    pub struct ColourPane(ObjectSubclass<imp::ColourPane>)
        @extends gtk::Widget, gtk::Box;
}

impl ColourPane {
    pub fn new(style: PaneStyle) -> Self {
        let pane = glib::Object::new::<Self>();
        pane.imp().style.set(style);
        pane.init();
        pane
    }

    pub fn set_hicolor(&self, project: &Project) {
        let imp = self.imp();
        let paintable_32 = project.paintable(crate::common::Icon::Scalable, Some(32));
        let paintable_64 = project.paintable(crate::common::Icon::Scalable, Some(64));
        let paintable_128 = project.paintable(crate::common::Icon::Scalable, Some(128));

        if let Some(icon) = imp.small_icons.borrow_mut().get(2) {
            icon.set_paintable(&project.name(), paintable_32.as_ref());
        }
        if let Some(icon) = imp.grid_icons.borrow_mut().get(1) {
            icon.set_paintable(&project.name(), paintable_64.as_ref());
        }

        imp.hicolor_128.set_from_paintable(paintable_128.as_ref());
        imp.hicolor_64.set_from_paintable(paintable_64.as_ref());
        imp.hicolor_32.set_from_paintable(paintable_32.as_ref());
    }

    pub fn set_symbolic(&self, project: &Project) {
        let imp = self.imp();

        match project.paintable(crate::common::Icon::Symbolic, None) {
            Some(paintable) => {
                imp.symbolic_image.set_from_paintable(Some(&paintable));
                imp.symbolic_image.set_visible(true);
                imp.symbolic_label.set_visible(true);
            }
            None => {
                imp.symbolic_image.set_visible(false);
                imp.symbolic_label.set_visible(false);
            }
        }
    }

    pub fn load_samples(&self, samples: &[gio::File]) {
        let imp = self.imp();
        // We fill the small icons
        assert_eq!(samples.len(), 6);
        let mut sample_idx = 0;
        for i in 0..5 {
            if i == 2 {
                continue;
            }
            let file = samples.get(sample_idx).unwrap();
            if let Some(icon) = imp.small_icons.borrow_mut().get(i) {
                icon.set_file(file);
            }
            sample_idx += 1;
        }
        // Then the grid ones
        let mut sample_idx = 4;
        for i in 0..3 {
            if i == 1 {
                continue;
            }
            let file = samples.get(sample_idx).unwrap();
            if let Some(icon) = imp.grid_icons.borrow_mut().get(i) {
                icon.set_file(file);
            }
            sample_idx += 1;
        }
    }

    fn init(&self) {
        let imp = self.imp();
        match imp.style.get() {
            PaneStyle::Dark => {
                self.add_css_class("dark");
            }
            PaneStyle::Light => {
                self.add_css_class("light");
            }
        };

        // Small container is composed of 5 icons, 4 samples & the previewed project
        let small_group = gtk::SizeGroup::new(gtk::SizeGroupMode::Horizontal);
        for _ in 0..5 {
            let demo_icon = Icon::new(IconSize::Small);
            small_group.add_widget(&demo_icon);
            imp.small.append(&demo_icon);
            imp.small_icons.borrow_mut().push(demo_icon);
        }

        // Grid container is composed of 3 icons, 2 samples & the previewed project
        let grid_group = gtk::SizeGroup::new(gtk::SizeGroupMode::Horizontal);
        for _ in 0..3 {
            let demo_icon = Icon::new(IconSize::Large);
            grid_group.add_widget(&demo_icon);
            imp.grid.append(&demo_icon);
            imp.grid_icons.borrow_mut().push(demo_icon);
        }
    }
}
