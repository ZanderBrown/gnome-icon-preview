use gtk::{
    gio,
    glib::{self, clone},
    prelude::*,
    subclass::prelude::*,
};

use super::item::RecentItemRow;
use crate::project::Project;

mod imp {
    use glib::subclass::Signal;

    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(resource = "/org/gnome/design/AppIconPreview/recents_popover.ui")]
    pub struct RecentsPopover {
        model: gtk::StringList,
        #[template_child]
        pub items_listbox: TemplateChild<gtk::ListBox>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for RecentsPopover {
        const NAME: &'static str = "RecentsPopover";
        type Type = super::RecentsPopover;
        type ParentType = gtk::Popover;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for RecentsPopover {
        fn signals() -> &'static [Signal] {
            use std::sync::OnceLock;
            static SIGNALS: OnceLock<Vec<Signal>> = OnceLock::new();
            SIGNALS.get_or_init(|| {
                vec![
                    Signal::builder("selected")
                        .param_types([Project::static_type()])
                        .build(),
                ]
            })
        }

        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();
            self.items_listbox.bind_model(
                Some(&self.model),
                clone!(
                    #[weak(rename_to = popover)]
                    obj,
                    #[upgrade_or_panic]
                    move |item| {
                        let item = item.downcast_ref::<gtk::StringObject>().unwrap();
                        let project =
                            Project::parse(gio::File::for_uri(&item.string()), false).unwrap();
                        let row = RecentItemRow::new(project.clone());

                        let gesture = gtk::GestureClick::new();
                        gesture.connect_released(clone!(
                            #[weak]
                            project,
                            #[weak]
                            popover,
                            move |gesture, _, _, _| {
                                popover.emit_by_name::<()>("selected", &[&project]);
                                popover.popdown();
                                gesture.set_state(gtk::EventSequenceState::Claimed);
                            }
                        ));
                        row.add_controller(gesture);
                        row.upcast::<gtk::Widget>()
                    }
                ),
            );

            let manager = gtk::RecentManager::default();
            let model = self.model.clone();
            let on_manager_changed = move |manager: &gtk::RecentManager| {
                manager.items().into_iter().for_each(clone!(
                    #[strong]
                    model,
                    move |item| {
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
                        if !exist_already && Project::parse(file, false).is_ok() {
                            model.append(&uri);
                        }
                    }
                ));
            };

            on_manager_changed(&manager);
            manager.connect_changed(on_manager_changed);
        }
    }
    impl WidgetImpl for RecentsPopover {}
    impl PopoverImpl for RecentsPopover {}
}

glib::wrapper! {
    pub struct RecentsPopover(ObjectSubclass<imp::RecentsPopover>)
        @extends gtk::Widget, gtk::Popover;
}

impl Default for RecentsPopover {
    fn default() -> Self {
        glib::Object::new()
    }
}
