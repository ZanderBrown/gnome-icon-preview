using Gtk;

namespace IconPreview {
	public class Colour : ScrolledWindow, Previewer {
		const string RES_PATH = "/org/gnome/IconPreview/icons/";
		const string BASE_THEME = "Adwaita";
		static string[] colours;

		private Box container = new Box(HORIZONTAL, 0);
		private ColourPane light = new ColourPane();
		private ColourPane dark = new ColourPane();

		private File _icon;
		public File previewing {
			get {
				return _icon;
			}
			set {
				_icon = value;
				light.icon = dark.icon = new FileIcon(_icon);
			}
		}

		private ColourExporter _export = new ColourExporter();
		public Exporter exporter {
			owned get {
				return _export;
			}
		}

		class construct {
			set_css_name("colour-view");
		}

		static construct {
			try {
				colours = resources_enumerate_children(RES_PATH, NONE);
			} catch (Error e) {
				critical("Failed to load sample icons: %s", e.message);
			}
		}

		construct {
			hscrollbar_policy = NEVER;
			min_content_height = 520;

			light.theme = BASE_THEME;
			dark.theme = BASE_THEME + "-dark";

			var view = new Viewport(null, null);
			view.shadow_type = NONE;
			view.show();

			container.add(light);
			container.add(dark);
			container.show();
			view.add(container);

			add(view);

			bind_property("previewing", _export, "file");

			shuffle();
		}

		public void shuffle () {
			var samples_names = random_selection(colours, 9);
			var samples = new Icon[9];

			for (var j = 0; j < 9; j++) {
				samples[j] = new FileIcon(File.new_for_uri("resource:/" + RES_PATH + samples_names[j]));
			}

			light.load_samples(samples);
			dark.load_samples(samples);
		}

		public Gdk.Pixbuf screenshot () {
			var w = container.get_allocated_width();
			var h = container.get_allocated_height();
			var surface = new Cairo.ImageSurface (ARGB32, w, h);
			var context = new Cairo.Context (surface);

			container.draw(context);

			Gdk.Pixbuf logo;
			try {
				logo = new Gdk.Pixbuf.from_resource_at_scale ("/org/gnome/IconPreview/badge.svg", 32, -1, true);
			} catch (Error e) {
				critical (e.message);
				logo = new Gdk.Pixbuf (RGB, false, 1, 2, 2);
			}
			var layout = container.create_pango_layout (_("Icon Preview"));

			var padding = 8;

			var img_height = logo.get_height();
			var img_width = logo.get_width();
			Pango.Rectangle txt_extents;

			layout.get_pixel_extents(null, out txt_extents);

			var img_x = 0;
			var txt_x = img_width + padding;
			if (container.get_direction () == RTL) {
				img_x = txt_extents.width + padding;
				txt_x = 0;
			}

			var img_y = 0;
			var txt_y = 0;
			if (txt_extents.height < img_height) {
				txt_y = (img_height - txt_extents.height) / 2;
			} else {
				img_y = (txt_extents.height - img_height) / 2;
			}

			context.save();
			Gdk.cairo_set_source_pixbuf (context, logo,
										 padding + img_x, padding + img_y);
			context.rectangle (padding + img_x, padding + img_y,
							   img_width, img_height);
			context.fill();
			context.restore();

			context.move_to (padding + txt_x, padding + txt_y);
			Pango.cairo_show_layout (context, layout);

			return Gdk.pixbuf_get_from_surface (surface, 0, 0, w, h);
		}
	}
}
