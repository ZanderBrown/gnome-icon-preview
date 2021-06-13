use super::colour_pane::{ColourPane, PaneStyle};
use crate::project::Project;

use gettextrs::gettext;
use rand::seq::SliceRandom;

use cairo;
use gtk::prelude::*;
use gtk::{gdk, gio, pango};
use pangocairo;
// A struct that represents a widget to render a Project
//
#[derive(Clone)]
pub struct ProjectPreviewer {
    pub widget: gtk::Box,
    light_panel: ColourPane,
    dark_panel: ColourPane,
    samples: Vec<String>,
}

impl ProjectPreviewer {
    pub fn new() -> Self {
        let widget = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        let light_panel = ColourPane::new(PaneStyle::Light);
        let dark_panel = ColourPane::new(PaneStyle::Dark);
        let samples = gio::resources_enumerate_children("/org/gnome/design/AppIconPreview/icons/", gio::ResourceLookupFlags::NONE)
            .unwrap()
            .iter()
            .map(|sample| sample.to_string())
            .filter(|sample| !sample.contains("-symbolic"))
            .collect::<Vec<String>>();

        let previewer = Self {
            widget,
            light_panel,
            dark_panel,
            samples,
        };
        previewer.init();
        previewer
    }

    pub fn screenshot(&self) -> Option<gdk_pixbuf::Pixbuf> {
        let width = self.widget.get_allocated_width();
        let height = self.widget.get_allocated_height();

        let surface = cairo::ImageSurface::create(cairo::Format::ARgb32, width, height).unwrap();
        let logo = gdk_pixbuf::Pixbuf::new_from_resource_at_scale("/org/gnome/design/AppIconPreview/badge.svg", 16, -1, true).unwrap();
        let layout = self.widget.create_pango_layout(Some(&gettext("App Icon Preview")))?;
        let mut font_description = pango::FontDescription::new();
        font_description.set_weight(pango::Weight::Semibold);
        font_description.set_size(pango::SCALE * 10);
        layout.set_font_description(Some(&font_description));

        let logo_width = logo.get_width();
        let logo_height = logo.get_height();
        let padding = 12.0;

        let (_, txt_extents) = layout.get_pixel_extents();

        let context = cairo::Context::new(&surface);

        self.widget.draw(&context);

        let mut img_x = 0.0;
        let txt_x = if self.widget.get_direction() == gtk::TextDirection::Rtl {
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
        context.save();

        context.set_source_pixbuf(&logo, padding + img_x, padding + img_y);
        context.rectangle(padding + img_x, padding + img_y, logo_width as f64, logo_height as f64);
        context.fill();
        context.restore();

        context.move_to(padding + txt_x, padding + txt_y);
        pangocairo::show_layout(&context, &layout);

        gdk::pixbuf_get_from_surface(&surface, 0, 0, width, height)
    }

    pub fn preview(&self, project: &Project) {
        if let Ok((hicolor, _)) = project.get_hicolor(None) {
            self.dark_panel.set_hicolor(&hicolor);
            self.light_panel.set_hicolor(&hicolor);
        }
        match project.get_symbolic() {
            Ok((symbolic, _)) => {
                self.light_panel.set_symbolic(Some(&symbolic));
                self.dark_panel.set_symbolic(Some(&symbolic));
            }
            Err(_) => {
                self.light_panel.set_symbolic(None);
                self.dark_panel.set_symbolic(None);
            }
        }
    }

    pub fn shuffle_samples(&self) {
        let mut rng = &mut rand::thread_rng();

        let samples = self
            .samples
            .choose_multiple(&mut rng, 6)
            .map(|sample_name| {
                let resource_uri = format!("resource://org/gnome/design/AppIconPreview/icons/{}", sample_name);
                gio::File::for_uri(&resource_uri)
            })
            .collect::<Vec<gio::File>>();

        self.light_panel.load_samples(&samples);
        self.dark_panel.load_samples(&samples);
    }

    fn init(&self) {
        self.widget.prepend(&self.light_panel.widget);
        self.widget.prepend(&self.dark_panel.widget);

        self.widget.add_css_class("previewer");
        self.shuffle_samples();
    }
}
