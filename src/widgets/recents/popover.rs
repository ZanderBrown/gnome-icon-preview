use super::item::RecentItemRow;
use gio::prelude::*;
use glib::Sender;
use gtk::prelude::*;

use crate::application::Action;
use crate::object_wrapper::ObjectWrapper;
use crate::project::Project;
use crate::widgets::recents::RecentItem;

pub struct RecentsPopover {
    pub widget: gtk::Popover,
    builder: gtk::Builder,
    sender: Sender<Action>,
    model: gio::ListStore,
}

impl RecentsPopover {
    pub fn new(sender: Sender<Action>) -> Self {
        let builder = gtk::Builder::new_from_resource("/org/gnome/design/AppIconPreview/recents_popover.ui");
        get_widget!(builder, gtk::Popover, recents_popover);

        let model = gio::ListStore::new(ObjectWrapper::static_type());
        let recents = Self {
            widget: recents_popover,
            builder,
            sender,
            model,
        };
        recents.init();
        recents
    }

    fn init(&self) {
        get_widget!(self.builder, gtk::ListBox, items_listbox);

        items_listbox.bind_model(
            Some(&self.model),
            clone!(@strong self.sender as sender, @strong self.widget as popover => move |item| {
                let item: RecentItem = item.downcast_ref::<ObjectWrapper>().unwrap().deserialize();
                let project = Project::parse(gio::File::new_for_uri(&item.uri)).unwrap();
                let row = RecentItemRow::new(project.clone());

                row.event_box.connect_button_press_event(clone!(@strong project, @strong sender, @strong popover => move |_, _| {
                    send!(sender, Action::OpenProject(project.clone()));
                    popover.popdown();
                    gtk::Inhibit(false)
                }));
                row.widget.upcast::<gtk::Widget>()
            }),
        );

        if let Some(manager) = gtk::RecentManager::get_default() {
            let model = self.model.clone();
            let on_manager_changed = move |manager: &gtk::RecentManager| {
                manager.get_items().into_iter().for_each(clone!(@strong model => move |item| {
                    let uri = item.get_uri().unwrap().to_string();
                    let file = gio::File::new_for_uri(&uri);
                    let mut exist_already = false;
                    for i in 0..model.get_n_items() {
                        let current_item: RecentItem = model.get_object(i).unwrap()
                                                        .downcast_ref::<ObjectWrapper>().unwrap().deserialize();
                        if current_item.uri == uri {
                            exist_already = true;
                            break;
                        }
                    }
                    if  !exist_already && Project::parse(file).is_ok() {
                        let object = ObjectWrapper::new(Box::new(RecentItem { uri }));
                        model.append(&object);
                    }
                }));
            };

            on_manager_changed(&manager);
            manager.connect_changed(on_manager_changed);
        }
    }
}
