use super::colour_pane::{ColourPane, PaneStyle};
use crate::project::{Project, ProjectType};

use gettextrs::gettext;
use rand::seq::SliceRandom;
use std::path::PathBuf;

use adw::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{
    gdk, gio,
    glib::{self, clone},
    pango,
};

// A struct that represents a widget to render a Project
mod imp {
    use super::*;
    use adw::subclass::prelude::*;

    pub struct ProjectPreviewer {
        pub light_panel: ColourPane,
        pub dark_panel: ColourPane,
        pub samples: Vec<String>,
        pub toast_overlay: adw::ToastOverlay,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ProjectPreviewer {
        const NAME: &'static str = "ProjectPreviewer";
        type ParentType = adw::Bin;
        type Type = super::ProjectPreviewer;

        fn new() -> Self {
            let light_panel = ColourPane::new(PaneStyle::Light);
            let dark_panel = ColourPane::new(PaneStyle::Dark);
            let toast_overlay = adw::ToastOverlay::new();
            let samples = gio::resources_enumerate_children("/org/gnome/design/AppIconPreview/icons/", gio::ResourceLookupFlags::NONE)
                .unwrap()
                .iter()
                .map(|sample| sample.to_string())
                .filter(|sample| !sample.contains("-symbolic"))
                .collect::<Vec<String>>();

            Self {
                light_panel,
                dark_panel,
                samples,
                toast_overlay,
            }
        }
    }
    impl ObjectImpl for ProjectPreviewer {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            let container = gtk::Box::new(gtk::Orientation::Horizontal, 0);
            container.append(&self.light_panel);
            container.append(&self.dark_panel);

            obj.set_child(Some(&self.toast_overlay));
            self.toast_overlay.set_child(Some(&container));

            obj.add_css_class("previewer");
            obj.shuffle_samples();
        }
    }
    impl WidgetImpl for ProjectPreviewer {}
    impl BinImpl for ProjectPreviewer {}
}

glib::wrapper! {
    pub struct ProjectPreviewer(ObjectSubclass<imp::ProjectPreviewer>)
        @extends adw::Bin, gtk::Widget;
}

impl ProjectPreviewer {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        glib::Object::new(&[]).unwrap()
    }

    fn screenshot(&self) -> Option<gdk_pixbuf::Pixbuf> {
        let width = self.allocated_width();
        let height = self.allocated_height();
        let scale = self.scale_factor();

        let surface = cairo::ImageSurface::create(cairo::Format::ARgb32, width * scale, height * scale).unwrap();
        surface.set_device_scale(scale as f64, scale as f64);

        let logo = gio::File::for_uri("resource:///org/gnome/design/AppIconPreview/badge.svg");
        let handle = rsvg::Loader::new().read_file(&logo, gio::Cancellable::NONE).ok()?;
        let renderer = rsvg::CairoRenderer::new(&handle);

        let layout = self.create_pango_layout(Some(&gettext("App Icon Preview")));
        let mut font_description = pango::FontDescription::new();
        font_description.set_weight(pango::Weight::Semibold);
        font_description.set_size(pango::SCALE * 10);
        layout.set_font_description(Some(&font_description));

        let dimensions = renderer.intrinsic_dimensions();
        let logo_width = dimensions.width.unwrap().length as i32;
        let logo_height = dimensions.height.unwrap().length as i32;

        let padding = 12.0;

        let (_, txt_extents) = layout.pixel_extents();

        let context = cairo::Context::new(&surface).ok()?;

        let snap = gtk::Snapshot::new();
        let paintable = gtk::WidgetPaintable::new(Some(self)).current_image().unwrap();
        paintable.snapshot(snap.upcast_ref::<gdk::Snapshot>(), width as f64, height as f64);
        let node = snap.to_node()?;
        node.draw(&context);

        let mut img_x = 0.0;
        let txt_x = if self.direction() == gtk::TextDirection::Rtl {
            img_x = txt_extents.width() as f64 + padding;
            0.0
        } else {
            logo_width as f64 + padding
        };

        let mut img_y = 0.0;
        let txt_y = if txt_extents.height() < logo_height {
            (logo_height - txt_extents.height()) as f64 / 2.0
        } else {
            img_y = (txt_extents.height() - logo_height) as f64 / 2.0;
            0.0
        };
        context.save().ok()?;
        renderer
            .render_document(
                &context,
                &cairo::Rectangle {
                    x: padding + img_x,
                    y: padding + img_y,
                    width: logo_width as f64,
                    height: logo_height as f64,
                },
            )
            .ok()?;
        context.fill().ok()?;
        context.restore().ok()?;

        context.move_to(padding + txt_x, padding + txt_y);
        pangocairo::show_layout(&context, &layout);

        gdk::pixbuf_get_from_surface(&surface, 0, 0, width * scale, height * scale)
    }

    pub fn preview(&self, project: &Project) {
        let imp = self.imp();

        imp.dark_panel.set_hicolor(&project.name());
        imp.light_panel.set_hicolor(&project.name());

        let symbolic = match project.project_type() {
            ProjectType::Icon => Some(project.name()),
            ProjectType::Preview => None,
        };

        imp.light_panel.set_symbolic(symbolic.as_deref());
        imp.dark_panel.set_symbolic(symbolic.as_deref());
    }

    pub fn shuffle_samples(&self) {
        let imp = self.imp();
        let mut rng = &mut rand::thread_rng();

        let samples = imp
            .samples
            .choose_multiple(&mut rng, 6)
            .map(|sample_name| {
                let resource_uri = format!("resource://org/gnome/design/AppIconPreview/icons/{}", sample_name);
                gio::File::for_uri(&resource_uri)
            })
            .collect::<Vec<gio::File>>();

        imp.light_panel.load_samples(&samples);
        imp.dark_panel.load_samples(&samples);
    }

    pub fn copy_screenshot(&self) {
        let display = gdk::Display::default().unwrap();
        let clipboard = display.clipboard();

        let pixbuf = self.screenshot().unwrap();

        let texture = gdk::Texture::for_pixbuf(&pixbuf);
        clipboard.set_texture(&texture);

        let toast = adw::Toast::new(&gettext("Screenshot copied to clipboard"));
        toast.set_timeout(3);
        self.imp().toast_overlay.add_toast(&toast);
    }

    pub fn save_screenshot(&self) {
        let pixbuf = self.screenshot().unwrap();
        let root = self.root().unwrap();

        let dialog = gtk::FileChooserNative::builder()
            .title(&gettext("Save Screenshot"))
            .modal(true)
            .accept_label(&gettext("_Save"))
            .cancel_label(&gettext("_Cancel"))
            .action(gtk::FileChooserAction::Save)
            .transient_for(root.downcast_ref::<gtk::Window>().unwrap())
            .build();
        dialog.set_current_name(&format!("{}.png", &gettext("Preview")));

        let xdg_pictures_dir = glib::user_special_dir(glib::UserDirectory::Pictures).unwrap();
        let gdir = gio::File::for_path(&xdg_pictures_dir);
        dialog.set_current_folder(&gdir).unwrap();

        let any_filter = gtk::FileFilter::new();
        any_filter.set_name(Some(&gettext("App Icon Preview")));
        any_filter.add_pattern("*.png");
        any_filter.add_mime_type("image/png");
        any_filter.add_pattern("*.jpg");
        any_filter.add_pattern("*.jpeg");
        any_filter.add_mime_type("image/jpeg");
        dialog.add_filter(&any_filter);

        let png_filter = gtk::FileFilter::new();
        png_filter.set_name(Some(&gettext("PNG")));
        png_filter.add_pattern("*.png");
        png_filter.add_mime_type("image/png");
        dialog.add_filter(&png_filter);

        let jpeg_filter = gtk::FileFilter::new();
        jpeg_filter.set_name(Some(&gettext("JPEG")));
        jpeg_filter.add_pattern("*.jpg");
        jpeg_filter.add_pattern("*.jpeg");
        jpeg_filter.add_mime_type("image/jpeg");
        dialog.add_filter(&jpeg_filter);

        dialog.connect_response(clone!(@strong pixbuf, @strong dialog => move |_, response| {
            if response == gtk::ResponseType::Accept {
                let filename: PathBuf = dialog.file().unwrap().basename().unwrap();
                let ext = match filename.extension() {
                    Some(ext) => ext.to_str().unwrap(),
                    None => "png"
                };
                let file = dialog.file().unwrap();
                let stream = file.replace(None, false,
                                          gio::FileCreateFlags::REPLACE_DESTINATION,
                                          gio::Cancellable::NONE).unwrap();

                pixbuf.save_to_streamv(&stream, ext, &[], gio::Cancellable::NONE).unwrap();
            }
            dialog.destroy();
        }));
        dialog.show();
    }
}
