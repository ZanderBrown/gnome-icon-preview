use super::colour_pane::{ColourPane, PaneStyle};
use crate::project::{Project, ProjectType};

use gettextrs::gettext;
use rand::seq::SliceRandom;

use adw::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gdk, gio, glib, graphene, gsk, pango};

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

    fn screenshot(&self) -> Option<gdk::Texture> {
        let width = self.allocated_width() as f32;
        let height = self.allocated_height() as f32;

        let padding: f32 = 12.0;
        let margin: f32 = 6.0;

        let logo = gdk::Texture::from_resource("/org/gnome/design/AppIconPreview/badge.svg");

        let logo_width = logo.intrinsic_width() as f32;
        let logo_height = logo.intrinsic_height() as f32;

        let layout = self.create_pango_layout(Some(&gettext("App Icon Preview")));
        let mut font_description = pango::FontDescription::new();
        font_description.set_weight(pango::Weight::Semibold);
        font_description.set_size(pango::SCALE * 10);
        layout.set_font_description(Some(&font_description));

        let (_, txt_extents) = layout.pixel_extents();
        let text_width = txt_extents.width() as f32;
        let text_height = txt_extents.height() as f32;

        let snap = gtk::Snapshot::new();

        // Compute relative positions.
        let logo_x = if self.direction() == gtk::TextDirection::Ltr { 0.0 } else { text_width + margin };
        let logo_y = 0.0;

        let txt_x = if self.direction() == gtk::TextDirection::Ltr { logo_width + margin } else { 0.0 };
        let txt_y = if text_height < logo_height { (logo_height - text_height) / 2.0 } else { 0.0 };

        // Snapshot previewer.
        let paintable = gtk::WidgetPaintable::new(Some(self)).current_image();
        paintable.snapshot(snap.upcast_ref::<gdk::Snapshot>(), width as f64, height as f64);

        // Snapshot logo.
        let origin = if self.direction() == gtk::TextDirection::Ltr {
            graphene::Point::new(padding, padding)
        } else {
            graphene::Point::new(width - padding - logo_width - margin - text_width, padding)
        };
        snap.translate(&origin);

        let point = graphene::Point::new(logo_x, logo_y);

        snap.save();
        snap.translate(&point);
        logo.snapshot(snap.upcast_ref::<gdk::Snapshot>(), logo_width as f64, logo_height as f64);
        snap.restore();

        // Snapshot text.
        let point = graphene::Point::new(txt_x, txt_y);
        snap.translate(&point);

        snap.append_layout(&layout, &gdk::RGBA::BLACK);

        // To texture
        let node = snap.to_node()?;
        let renderer = gsk::GLRenderer::new();
        renderer.realize(gdk::Surface::NONE).ok()?;
        let texture = renderer.render_texture(&node, None);
        renderer.unrealize();

        Some(texture)
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

        let texture = self.screenshot().unwrap();

        clipboard.set_texture(&texture);

        let toast = adw::Toast::new(&gettext("Screenshot copied to clipboard"));
        toast.set_timeout(3);
        self.imp().toast_overlay.add_toast(&toast);
    }

    pub async fn save_screenshot(&self) -> anyhow::Result<()> {
        let texture = self.screenshot().unwrap();
        let bytes = texture.save_to_png_bytes();
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

        if dialog.run_future().await == gtk::ResponseType::Accept {
            let file = dialog.file().unwrap();

            let stream = file.replace_future(None, false, gio::FileCreateFlags::REPLACE_DESTINATION, glib::PRIORITY_DEFAULT).await?;
            stream.write_bytes_future(&bytes, glib::PRIORITY_DEFAULT).await?;
        };
        dialog.destroy();

        Ok(())
    }
}
