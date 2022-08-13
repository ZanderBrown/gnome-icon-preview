use crate::application::Action;

use gettextrs::gettext;
use log::error;
use std::iter::FromIterator;
use std::path::PathBuf;

use gtk::glib::clone;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gdk, gio, glib};
use gtk_macros::send;

mod imp {
    use super::*;
    use adw::subclass::prelude::*;
    use once_cell::sync::OnceCell;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(resource = "/org/gnome/design/AppIconPreview/new_project.ui")]
    pub struct NewProjectDialog {
        pub sender: OnceCell<glib::Sender<Action>>,
        #[template_child]
        pub project_name: TemplateChild<gtk::Entry>,
        #[template_child]
        pub project_path: TemplateChild<gtk::Entry>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for NewProjectDialog {
        const NAME: &'static str = "NewProjectDialog";
        type Type = super::NewProjectDialog;
        type ParentType = adw::Window;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            klass.install_action("project.cancel", None, |widget, _, _| {
                widget.destroy();
            });
            klass.install_action("project.create", None, |widget, _, _| {
                let imp = widget.imp();

                let project_name = format!("{}.Source.svg", imp.project_name.text());
                let project_path = imp.project_path.text().replacen("~", glib::home_dir().to_str().unwrap(), 1);

                let dest_path = PathBuf::from_iter(&[project_path, project_name]);

                let project_file = gio::File::for_path(&dest_path);

                let sender = imp.sender.get().unwrap();
                send!(sender, Action::NewProject(project_file));
                widget.destroy();
            });
            klass.install_action("project.browse", None, |parent, _, _| {
                let dialog = gtk::FileChooserDialog::new(
                    Some(&gettext("Select Icon Location")),
                    Some(parent),
                    gtk::FileChooserAction::SelectFolder,
                    &[(&gettext("Select"), gtk::ResponseType::Accept), (&gettext("Cancel"), gtk::ResponseType::Cancel)],
                );
                dialog.set_default_response(gtk::ResponseType::Accept);
                dialog.set_modal(true);
                let home_dir = gio::File::for_path(&glib::home_dir());
                dialog.set_current_folder(Some(&home_dir)).unwrap();
                dialog.connect_response(clone!(@weak dialog, @weak parent => move |_, response| {
                    if response == gtk::ResponseType::Accept {
                        let home = glib::home_dir();
                        let home = home.to_str().unwrap();

                        let dest = dialog.file().unwrap().path().unwrap();
                        let dest = dest.to_str().unwrap();
                        let dest = dest.replacen(&home, "~", 1);

                        parent.imp().project_path.set_text(&dest);
                    }
                    dialog.destroy();
                }));
                dialog.show();
            });
            klass.add_binding_action(gdk::Key::Escape, gdk::ModifierType::empty(), "window.close", None);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for NewProjectDialog {}
    impl WidgetImpl for NewProjectDialog {}
    impl WindowImpl for NewProjectDialog {}
    impl AdwWindowImpl for NewProjectDialog {}
}

glib::wrapper! {
    pub struct NewProjectDialog(ObjectSubclass<imp::NewProjectDialog>)
        @extends gtk::Widget, gtk::Window, adw::Window;
}

impl NewProjectDialog {
    pub fn new(sender: glib::Sender<Action>) -> Self {
        let dialog = glib::Object::new::<Self>(&[]).unwrap();
        dialog.imp().sender.set(sender).unwrap();
        dialog.init();
        dialog
    }

    fn init(&self) {
        self.action_set_enabled("project.create", false);
        self.imp().project_name.connect_changed(clone!(@weak self as dialog => move |entry| {
            let app_id = entry.text().to_string();
            dialog.action_set_enabled("project.create", gio::Application::id_is_valid(&app_id));
        }));
    }
}
