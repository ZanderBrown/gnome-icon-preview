use crate::application::Action;
use crate::common;

use gettextrs::gettext;
use std::iter::FromIterator;
use std::path::PathBuf;

use gtk::glib::clone;
use gtk::prelude::*;
use gtk::{gio, glib};
use gtk_macros::get_widget;

pub struct NewProjectDialog {
    pub widget: adw::Window,
    builder: gtk::Builder,
    sender: glib::Sender<Action>,
}

impl NewProjectDialog {
    pub fn new(sender: glib::Sender<Action>) -> Self {
        let builder = gtk::Builder::from_resource("/org/gnome/design/AppIconPreview/new_project.ui");
        get_widget!(builder, adw::Window, dialog);

        let new_project = Self { widget: dialog, builder, sender };
        new_project.init();
        new_project
    }

    fn init(&self) {
        get_widget!(self.builder, gtk::Button, cancel_button);
        cancel_button.connect_clicked(clone!(@weak self.widget as dialog => move |_| {
            dialog.destroy();
        }));

        get_widget!(self.builder, gtk::Button, accept_button);
        accept_button.connect_clicked(clone!(@strong self.sender as sender, @strong self.builder as builder, @weak self.widget as dialog => move |_| {
            let project_name = get_widget!(builder, gtk::Entry, @project_name).text();
            let project_name = format!("{}.Source.svg", project_name);
            let project_path = get_widget!(builder, gtk::Entry, @project_path).text();
            let project_path = project_path.as_str().replacen("~", glib::home_dir().to_str().unwrap(), 1);

            let dest_path = PathBuf::from_iter(&[project_path, project_name]);

            let project_file = gio::File::for_path(&dest_path);

            send!(sender, Action::NewProject(project_file));
            dialog.destroy();

        }));

        get_widget!(self.builder, gtk::Button, browse_button);
        browse_button.connect_clicked(clone!(@weak self.widget as parent, @strong self.builder as builder => move |_| {
            let dialog = gtk::FileChooserDialog::new(None, Some(&parent), gtk::FileChooserAction::SelectFolder,
                &[(&gettext("Select"), gtk::ResponseType::Accept),
                (&gettext("Cancel"), gtk::ResponseType::Cancel)]
            );
            let home_dir = gio::File::for_path(&glib::home_dir());
            dialog.set_current_folder(&home_dir);
            dialog.connect_response(clone!(@strong builder, @strong dialog => move |_, response| {

                if response == gtk::ResponseType::Accept {
                    get_widget!(builder, gtk::Entry, project_path);
                    let home = glib::home_dir();
                    let home = home.to_str().unwrap();

                    let dest = dialog.file().unwrap().path().unwrap();
                    let dest = dest.to_str().unwrap();
                    let dest = dest.replacen(&home, "~", 1);

                    project_path.set_text(&dest);
                }
                dialog.destroy();
            }));
            dialog.show();
        }));

        get_widget!(self.builder, gtk::Entry, project_name);
        project_name.connect_changed(clone!(@strong self.builder as builder => move |entry| {
            get_widget!(builder, gtk::Button, accept_button);
            let app_id = entry.text().to_string();
            accept_button.set_sensitive(common::is_valid_app_id(&app_id));
        }));
    }
}
