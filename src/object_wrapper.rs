// ObjectWrapper is a GObject subclass, which we need to carry the rustio::Station/song::Song struct.
// With this we can use gtk::ListBox bind_model() properly.
//
// For more details, you should look at this gtk-rs example:
// https://github.com/gtk-rs/examples/blob/master/src/bin/listbox_model.rs
// Source https://gitlab.gnome.org/World/Shortwave/blob/master/src/model/object_wrapper.rs

use serde::de::DeserializeOwned;

use gtk::glib;
use gtk::glib::subclass::prelude::*;
use gtk::prelude::*;

mod imp {
    use super::*;

    use once_cell::sync::Lazy;
    use std::cell::RefCell;

    pub struct ObjectWrapper {
        data: RefCell<Option<String>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ObjectWrapper {
        const NAME: &'static str = "ObjectWrapper";
        type ParentType = glib::Object;
        type Type = super::ObjectWrapper;

        fn new() -> Self {
            Self { data: RefCell::new(None) }
        }
    }

    impl ObjectImpl for ObjectWrapper {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| vec![glib::ParamSpec::new_string("data", "Data", "Data", None, glib::ParamFlags::READWRITE)]);
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _obj: &Self::Type, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            match pspec.name() {
                "data" => {
                    let data = value.get().expect("ObjectWrapper `data` getter");
                    self.data.replace(data);
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "data" => self.data.borrow().to_value(),
                _ => unimplemented!(),
            }
        }
    }
}

glib::wrapper! {
    pub struct ObjectWrapper(ObjectSubclass<imp::ObjectWrapper>);
}

impl ObjectWrapper {
    pub fn new<O>(object: O) -> ObjectWrapper
    where
        O: serde::ser::Serialize,
    {
        glib::Object::new(&[("data", &serde_json::to_string(&object).unwrap())]).unwrap()
    }

    pub fn deserialize<O>(&self) -> O
    where
        O: DeserializeOwned,
    {
        let data = self.property("data").unwrap().get::<String>().unwrap();
        serde_json::from_str(&data).unwrap()
    }
}
