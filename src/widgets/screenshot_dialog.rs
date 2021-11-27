use gettextrs::gettext;
use std::path::PathBuf;

use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{
    gdk, gio,
    glib::{self, clone},
};

mod imp {
    use super::*;
    use adw::subclass::prelude::*;
    use glib::{ParamSpec, Value};
    use once_cell::sync::{Lazy, OnceCell};

    #[derive(Default, Debug, gtk::CompositeTemplate)]
    #[template(resource = "/org/gnome/design/AppIconPreview/screenshot_dialog.ui")]
    pub struct ScreenshotDialog {
        pub pixbuf: OnceCell<gdk_pixbuf::Pixbuf>,
        #[template_child]
        pub preview: TemplateChild<gtk::Picture>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ScreenshotDialog {
        const NAME: &'static str = "ScreenshotDialog";
        type Type = super::ScreenshotDialog;
        type ParentType = adw::Window;

        fn class_init(klass: &mut Self::Class) {
            klass.install_action("screenshot.copy", None, |widget, _, _| {
                widget.copy();
            });
            klass.install_action("screenshot.save", None, |widget, _, _| {
                widget.save();
            });
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ScreenshotDialog {
        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![ParamSpec::new_object(
                    "pixbuf",
                    "Pixbuf",
                    "The widget's pixbuf",
                    gdk_pixbuf::Pixbuf::static_type(),
                    glib::ParamFlags::READWRITE,
                )]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _obj: &Self::Type, _id: usize, value: &Value, pspec: &ParamSpec) {
            match pspec.name() {
                "pixbuf" => {
                    let pixbuf = value.get().unwrap();
                    self.pixbuf.set(pixbuf).unwrap();
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> Value {
            match pspec.name() {
                "pixbuf" => self.pixbuf.get().to_value(),
                _ => unimplemented!(),
            }
        }
    }
    impl WidgetImpl for ScreenshotDialog {}
    impl WindowImpl for ScreenshotDialog {}
    impl AdwWindowImpl for ScreenshotDialog {}
}

glib::wrapper! {
    pub struct ScreenshotDialog(ObjectSubclass<imp::ScreenshotDialog>)
        @extends gtk::Widget, gtk::Window, adw::Window;
}

impl ScreenshotDialog {
    pub fn new(pixbuf: gdk_pixbuf::Pixbuf) -> Self {
        let dialog = glib::Object::new::<Self>(&[("pixbuf", &pixbuf)]).unwrap();
        dialog.init();
        dialog
    }

    fn init(&self) {
        let self_ = imp::ScreenshotDialog::from_instance(self);
        let pixbuf = self_.pixbuf.get().unwrap();

        let aspect_ratio = pixbuf.width() as f32 / pixbuf.height() as f32;
        let width = 600;
        let height = (width as f32 / aspect_ratio) as i32;
        let scaled_pixbuf = pixbuf.scale_simple(width, height, gdk_pixbuf::InterpType::Bilinear);

        self_.preview.set_pixbuf(scaled_pixbuf.as_ref());
    }

    pub fn copy(&self) {
        let display = gdk::Display::default().unwrap();
        let clipboard = display.clipboard();

        let self_ = imp::ScreenshotDialog::from_instance(self);
        let pixbuf = self_.pixbuf.get().unwrap();

        let texture = gdk::Texture::for_pixbuf(&pixbuf);
        clipboard.set_texture(&texture);
    }

    fn save(&self) {
        let self_ = imp::ScreenshotDialog::from_instance(self);
        let pixbuf = self_.pixbuf.get().unwrap();

        let dialog = gtk::FileChooserNative::new(
            Some(&gettext("Save Screenshot")),
            Some(self),
            gtk::FileChooserAction::Save,
            Some(&gettext("_Save")),
            Some(&gettext("_Cancel")),
        );
        dialog.set_modal(true);
        dialog.set_current_name(&format!("{}.png", &gettext("Preview")));

        let xdg_pictures_dir = glib::user_special_dir(glib::UserDirectory::Pictures);
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
                                          gio::NONE_CANCELLABLE).unwrap();

                pixbuf.save_to_streamv(&stream, ext, &[], gio::NONE_CANCELLABLE).unwrap();
            }
            dialog.destroy();
        }));
        dialog.show();
    }
}
