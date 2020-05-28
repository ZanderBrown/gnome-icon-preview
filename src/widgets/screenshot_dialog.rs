use gettextrs::gettext;
use gio::prelude::*;
use gtk::prelude::*;
use std::path::PathBuf;
use std::rc::Rc;

pub struct ScreenshotDialog {
    pub widget: libhandy::Dialog,
    builder: gtk::Builder,
    actions: gio::SimpleActionGroup,
    pixbuf: gdk_pixbuf::Pixbuf,
}

impl ScreenshotDialog {
    pub fn new(pixbuf: gdk_pixbuf::Pixbuf) -> Rc<Self> {
        let builder = gtk::Builder::new_from_resource("/org/gnome/design/AppIconPreview/screenshot_dialog.ui");
        get_widget!(builder, libhandy::Dialog, screenshot_dialog);

        let previewer = Rc::new(Self {
            widget: screenshot_dialog,
            builder,
            pixbuf,
            actions: gio::SimpleActionGroup::new(),
        });

        previewer.init(previewer.clone());
        previewer
    }

    pub fn copy(&self) -> Option<()> {
        let display = gdk::Display::get_default()?;
        let clipboard = gtk::Clipboard::get_default(&display)?;

        clipboard.set_image(&self.pixbuf);
        Some(())
    }

    pub fn save(&self) {
        let dialog = gtk::FileChooserNative::new(
            Some(&gettext("Save Screenshot")),
            Some(&self.widget),
            gtk::FileChooserAction::Save,
            Some(&gettext("_Save")),
            Some(&gettext("_Cancel")),
        );
        dialog.set_modal(true);
        dialog.set_current_name(&gettext("Preview"));

        let xdg_pictures_dir = glib::get_user_special_dir(glib::UserDirectory::Pictures).unwrap();
        dialog.set_current_folder(&xdg_pictures_dir);

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

        dialog.connect_response(clone!(@strong self.pixbuf as pixbuf, @strong dialog => move |_, response| {
            if response == gtk::ResponseType::Accept {
                let filename: PathBuf = dialog.get_filename().unwrap();
                let ext = match filename.extension() {
                    Some(ext) => ext.to_str().unwrap(),
                    None => "png"
                };
                let file = dialog.get_file().unwrap();
                let stream = file.replace(None, false,
                                          gio::FileCreateFlags::REPLACE_DESTINATION,
                                          gio::NONE_CANCELLABLE).unwrap();

                pixbuf.save_to_streamv(&stream, ext, &[], gio::NONE_CANCELLABLE).unwrap();
            }
            dialog.destroy();
        }));
        dialog.show();
    }

    fn init(&self, rc_s: Rc<Self>) {
        let ratio = self.pixbuf.get_width() / 450;
        let height = (self.pixbuf.get_height() / ratio) as i32;
        let scaled_pixbuf = self.pixbuf.scale_simple(450, height, gdk_pixbuf::InterpType::Bilinear);
        get_widget!(self.builder, gtk::Image, @preview).set_from_pixbuf(scaled_pixbuf.as_ref());

        action!(
            self.actions,
            "copy",
            clone!(@strong rc_s as screenshot =>  move |_, _| {
                screenshot.copy();
            })
        );
        action!(
            self.actions,
            "save",
            clone!(@strong rc_s as screenshot =>  move |_, _| {
                screenshot.save();
            })
        );

        action!(
            self.actions,
            "close",
            clone!(@weak self.widget as dialog => move |_, _| {
                dialog.destroy();
            })
        );

        self.widget.insert_action_group("screenshot", Some(&self.actions));
    }
}
