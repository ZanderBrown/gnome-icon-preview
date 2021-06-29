use super::icon::Icon;

use std::cell::RefCell;

use gtk::gio;
use gtk::prelude::*;
use gtk_macros::get_widget;

#[derive(PartialEq, Clone)]
pub enum PaneStyle {
    Light,
    Dark,
}

#[derive(Clone)]
pub struct ColourPane {
    pub widget: gtk::Box,
    builder: gtk::Builder,
    style: PaneStyle,
    small_icons: RefCell<Vec<Icon>>,
    grid_icons: RefCell<Vec<Icon>>,
}

impl ColourPane {
    pub fn new(style: PaneStyle) -> Self {
        let builder = gtk::Builder::from_resource("/org/gnome/design/AppIconPreview/colourpane.ui");
        get_widget!(builder, gtk::Box, colour_pane);

        let pane = Self {
            widget: colour_pane,
            builder,
            style,
            small_icons: RefCell::new(Vec::new()),
            grid_icons: RefCell::new(Vec::new()),
        };
        pane.init();
        pane
    }

    pub fn set_hicolor(&self, hicolor: &gio::File) {
        if let Some(icon) = self.small_icons.borrow_mut().get(2) {
            icon.set_file(hicolor);
        }
        if let Some(icon) = self.grid_icons.borrow_mut().get(1) {
            icon.set_file(hicolor);
        }
        let gicon = gio::FileIcon::new(hicolor);

        get_widget!(self.builder, gtk::Image, hicolor_128);
        get_widget!(self.builder, gtk::Image, hicolor_64);
        get_widget!(self.builder, gtk::Image, hicolor_32);

        hicolor_128.set_from_gicon(&gicon);
        hicolor_64.set_from_gicon(&gicon);
        hicolor_32.set_from_gicon(&gicon);
    }

    pub fn set_symbolic(&self, symbolic: Option<&gio::File>) {
        get_widget!(self.builder, gtk::Image, symbolic_img);
        get_widget!(self.builder, gtk::Label, symbolic_label);

        match symbolic {
            Some(symbolic_file) => {
                symbolic_img.set_from_gicon(&gio::FileIcon::new(symbolic_file));
                symbolic_img.show();
                symbolic_label.show();
            }
            None => {
                symbolic_img.hide();
                symbolic_label.hide();
            }
        }
    }

    pub fn load_samples(&self, samples: &[gio::File]) {
        // We fill the small icons
        assert_eq!(samples.len(), 6);
        let mut sample_idx = 0;
        for i in 0..5 {
            if i == 2 {
                continue;
            }
            let file = samples.get(sample_idx).unwrap();
            if let Some(icon) = self.small_icons.borrow_mut().get(i) {
                icon.set_file(file);
            }
            sample_idx += 1;
        }
        // Then the grid ones
        let mut sample_idx = 4;
        for i in 0..3 {
            if i == 1 {
                continue;
            }
            let file = samples.get(sample_idx).unwrap();
            if let Some(icon) = self.grid_icons.borrow_mut().get(i) {
                icon.set_file(file);
            }
            sample_idx += 1;
        }
    }

    fn init(&self) {
        let css_provider = gtk::CssProvider::new();
        match self.style {
            PaneStyle::Dark => {
                self.widget.add_css_class("dark");
                css_provider.load_named("Adwaita", Some("dark"));
            }
            PaneStyle::Light => css_provider.load_named("Adwaita", None),
        };
        self.widget.style_context().add_provider(&css_provider, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);
        get_widget!(self.builder, gtk::Image, @symbolic_img)
            .style_context()
            .add_provider(&css_provider, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);

        // Small container is composed of 5 icons, 4 samples & the previewed project
        get_widget!(self.builder, gtk::Box, small);
        let small_group = gtk::SizeGroup::new(gtk::SizeGroupMode::Horizontal);
        for _ in 0..5 {
            let demo_icon = Icon::new(None, 64);
            small_group.add_widget(&demo_icon.widget);
            demo_icon.label.add_css_class("caption");
            small.append(&demo_icon.widget);
            self.small_icons.borrow_mut().push(demo_icon);
        }

        // Grid container is composed of 3 icons, 2 samples & the previewed project
        get_widget!(self.builder, gtk::Box, grid);
        let grid_group = gtk::SizeGroup::new(gtk::SizeGroupMode::Horizontal);
        for _ in 0..3 {
            let demo_icon = Icon::new(None, 96);
            grid_group.add_widget(&demo_icon.widget);
            grid.append(&demo_icon.widget);
            self.grid_icons.borrow_mut().push(demo_icon);
        }
    }
}
