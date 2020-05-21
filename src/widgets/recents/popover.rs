use super::item::RecentItemRow;
use glib::Sender;
use gtk::prelude::*;

use crate::application::Action;
use crate::object_wrapper::ObjectWrapper;
use crate::project::Project;
use crate::recents::RecentItem;

pub struct RecentsPopover {
    pub widget: gtk::Popover,
    builder: gtk::Builder,
    sender: Sender<Action>,
}

impl RecentsPopover {
    pub fn new(history_model: &gio::ListStore, sender: Sender<Action>) -> Self {
        let builder = gtk::Builder::new_from_resource("/org/gnome/design/AppIconPreview/recents_popover.ui");
        get_widget!(builder, gtk::Popover, recents_popover);
        let recents = Self {
            widget: recents_popover,
            builder,
            sender,
        };
        recents.init(history_model);
        recents
    }

    fn init(&self, history_model: &gio::ListStore) {
        get_widget!(self.builder, gtk::ListBox, items_listbox);

        items_listbox.bind_model(
            Some(history_model),
            clone!(@strong self.sender as sender, @strong self.widget as popover => move |item| {
                let item: RecentItem = item.downcast_ref::<ObjectWrapper>().unwrap().deserialize();
                let project = Project::parse(gio::File::new_for_path(item.path)).unwrap();
                let row = RecentItemRow::new(project.clone());

                row.event_box.connect_button_press_event(clone!(@strong project, @strong sender, @strong popover => move |_, _| {
                    send!(sender, Action::OpenProject(project.clone()));
                    popover.popdown();
                    gtk::Inhibit(false)
                }));
                row.widget.upcast::<gtk::Widget>()
            }),
        );
    }
}
