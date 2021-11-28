use super::item::RecentItemRow;
use crate::application::Action;
use crate::project::Project;

use log::error;

use gtk::glib::{clone, Sender};
use gtk::subclass::prelude::*;
use gtk::{gio, glib, prelude::*};
use gtk_macros::send;

mod imp {
    use super::*;
    use once_cell::sync::OnceCell;

    #[derive(Debug, gtk::CompositeTemplate)]
    #[template(resource = "/org/gnome/design/AppIconPreview/recents_popover.ui")]
    pub struct RecentsPopover {
        pub sender: OnceCell<Sender<Action>>,
        model: gtk::StringList,
        #[template_child]
        pub items_listbox: TemplateChild<gtk::ListBox>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for RecentsPopover {
        const NAME: &'static str = "RecentsPopover";
        type Type = super::RecentsPopover;
        type ParentType = gtk::Popover;

        fn new() -> Self {
            Self {
                model: gtk::StringList::new(&[]),
                items_listbox: Default::default(),
                sender: Default::default(),
            }
        }

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for RecentsPopover {
        fn constructed(&self, obj: &Self::Type) {
            self.items_listbox.bind_model(
                Some(&self.model),
                clone!(@weak obj as popover => @default-panic, move |item| {
                    let item = item.downcast_ref::<gtk::StringObject>().unwrap();
                    let project = Project::parse(gio::File::for_uri(&item.string()), false).unwrap();
                    let row = RecentItemRow::new(project.clone());

                    let gesture = gtk::GestureClick::new();
                    gesture.connect_released(clone!(@weak project, @weak popover => move |gesture, _, _, _| {
                        let self_ = imp::RecentsPopover::from_instance(&popover);

                        send!(self_.sender.get().unwrap(), Action::OpenProject(project));
                        popover.popdown();
                        gesture.set_state(gtk::EventSequenceState::Claimed);
                    }));
                    row.add_controller(&gesture);
                    row.upcast::<gtk::Widget>()
                }),
            );

            let manager = gtk::RecentManager::default();
            let model = self.model.clone();
            let on_manager_changed = move |manager: &gtk::RecentManager| {
                manager.items().into_iter().for_each(clone!(@strong model => move |item| {
                    let uri = item.uri().to_string();
                    let file = gio::File::for_uri(&uri);
                    let mut exist_already = false;
                    for i in 0..model.n_items() {
                        let current = model.item(i).unwrap();
                        let string_obj = current.downcast_ref::<gtk::StringObject>().unwrap();
                        if string_obj.string() == uri {
                            exist_already = true;
                            break;
                        }
                    }
                    if  !exist_already && Project::parse(file, false).is_ok() {
                        model.append(&uri);
                    }
                }));
            };

            on_manager_changed(&manager);
            manager.connect_changed(on_manager_changed);

            self.parent_constructed(obj);
        }
    }
    impl WidgetImpl for RecentsPopover {}
    impl PopoverImpl for RecentsPopover {}
}

glib::wrapper! {
    pub struct RecentsPopover(ObjectSubclass<imp::RecentsPopover>)
        @extends gtk::Widget, gtk::Popover;
}

impl RecentsPopover {
    pub fn new(sender: Sender<Action>) -> Self {
        let popover = glib::Object::new::<Self>(&[]).unwrap();
        let self_ = imp::RecentsPopover::from_instance(&popover);
        self_.sender.set(sender).unwrap();
        popover
    }
}
