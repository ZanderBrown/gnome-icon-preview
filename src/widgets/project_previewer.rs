use super::colour_pane::{ColourPane, PaneStyle};
use crate::project::Project;

use gettextrs::gettext;
use rand::seq::SliceRandom;

use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gdk, gio, glib, pango};
// A struct that represents a widget to render a Project
mod imp {
    use super::*;

    pub struct ProjectPreviewer {
        pub light_panel: ColourPane,
        pub dark_panel: ColourPane,
        pub samples: Vec<String>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ProjectPreviewer {
        const NAME: &'static str = "ProjectPreviewer";
        type ParentType = gtk::Box;
        type Type = super::ProjectPreviewer;

        fn new() -> Self {
            let light_panel = ColourPane::new(PaneStyle::Light);
            let dark_panel = ColourPane::new(PaneStyle::Dark);
            let samples = gio::resources_enumerate_children("/org/gnome/design/AppIconPreview/icons/", gio::ResourceLookupFlags::NONE)
                .unwrap()
                .iter()
                .map(|sample| sample.to_string())
                .filter(|sample| !sample.contains("-symbolic"))
                .collect::<Vec<String>>();

            Self { light_panel, dark_panel, samples }
        }
    }
    impl ObjectImpl for ProjectPreviewer {
        fn constructed(&self, obj: &Self::Type) {
            obj.append(&self.light_panel.widget);
            obj.append(&self.dark_panel.widget);

            obj.add_css_class("previewer");
            obj.shuffle_samples();

            self.parent_constructed(obj);
        }
    }
    impl WidgetImpl for ProjectPreviewer {}
    impl BoxImpl for ProjectPreviewer {}
}

glib::wrapper! {
    pub struct ProjectPreviewer(ObjectSubclass<imp::ProjectPreviewer>)
        @extends gtk::Box, gtk::Widget;
}

impl ProjectPreviewer {
    pub fn new() -> Self {
        glib::Object::new(&[]).unwrap()
    }

    pub fn screenshot(&self) -> Option<gdk_pixbuf::Pixbuf> {
        let width = self.allocated_width();
        let height = self.allocated_height();

        let surface = cairo::ImageSurface::create(cairo::Format::ARgb32, width, height).unwrap();
        let logo = gdk_pixbuf::Pixbuf::from_resource_at_scale("/org/gnome/design/AppIconPreview/badge.svg", 16, -1, true).unwrap();
        let layout = self.create_pango_layout(Some(&gettext("App Icon Preview")));
        let mut font_description = pango::FontDescription::new();
        font_description.set_weight(pango::Weight::Semibold);
        font_description.set_size(pango::SCALE * 10);
        layout.set_font_description(Some(&font_description));

        let logo_width = logo.width();
        let logo_height = logo.height();
        let padding = 12.0;

        let (_, txt_extents) = layout.pixel_extents();

        let context = cairo::Context::new(&surface).ok()?;

        let snap = gtk::Snapshot::new();
        let paintable = gtk::WidgetPaintable::new(Some(self));
        paintable.snapshot(&snap.upcast_ref::<gdk::Snapshot>(), width as f64, height as f64);
        let node = snap.free_to_node()?;
        node.draw(&context);

        let mut img_x = 0.0;
        let txt_x = if self.direction() == gtk::TextDirection::Rtl {
            img_x = txt_extents.width as f64 + padding;
            0.0
        } else {
            logo_width as f64 + padding
        };

        let mut img_y = 0.0;
        let txt_y = if txt_extents.height < logo_height {
            (logo_height - txt_extents.height) as f64 / 2.0
        } else {
            img_y = (txt_extents.height - logo_height) as f64 / 2.0;
            0.0
        };
        context.save().ok()?;

        context.set_source_pixbuf(&logo, padding + img_x, padding + img_y);
        context.rectangle(padding + img_x, padding + img_y, logo_width as f64, logo_height as f64);
        context.fill().ok()?;
        context.restore().ok()?;

        context.move_to(padding + txt_x, padding + txt_y);
        pangocairo::show_layout(&context, &layout);

        gdk::pixbuf_get_from_surface(&surface, 0, 0, width, height)
    }

    pub fn preview(&self, project: &Project) {
        let self_ = imp::ProjectPreviewer::from_instance(self);

        if let Ok((hicolor, _)) = project.get_hicolor(None) {
            self_.dark_panel.set_hicolor(&hicolor);
            self_.light_panel.set_hicolor(&hicolor);
        }
        match project.get_symbolic() {
            Ok((symbolic, _)) => {
                self_.light_panel.set_symbolic(Some(&symbolic));
                self_.dark_panel.set_symbolic(Some(&symbolic));
            }
            Err(_) => {
                self_.light_panel.set_symbolic(None);
                self_.dark_panel.set_symbolic(None);
            }
        }
    }
    pub fn shuffle_samples(&self) {
        let self_ = imp::ProjectPreviewer::from_instance(self);
        let mut rng = &mut rand::thread_rng();

        let samples = self_
            .samples
            .choose_multiple(&mut rng, 6)
            .map(|sample_name| {
                let resource_uri = format!("resource://org/gnome/design/AppIconPreview/icons/{}", sample_name);
                gio::File::for_uri(&resource_uri)
            })
            .collect::<Vec<gio::File>>();

        self_.light_panel.load_samples(&samples);
        self_.dark_panel.load_samples(&samples);
    }
}
