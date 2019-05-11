using Gtk;

namespace IconPreview {
	public class Colour : Box, Previewer {
		const string RES_PATH = "/org/gnome/IconPreview/icons/";
		const string BASE_THEME = "Adwaita";
		static string[] colours;

		private ColourPane light = new ColourPane();
		private ColourPane dark = new ColourPane();

		private File _icon;
		public File previewing {
			get {
				return _icon;
			}
			set {
				_icon = value;
				var svg = new Rsvg.Handle.from_gfile_sync(_icon, FLAGS_NONE);
				var hicolor = split_svg(svg, "#hicolor");
				var symbolic = split_svg(svg, "#symbolic");

				light.name = dark.name = _icon.get_basename();

				if (hicolor != null) {
					light.hicolor = dark.hicolor = hicolor;
				} else {
					light.hicolor = dark.hicolor = _icon;
				}
				light.symbolic = dark.symbolic = symbolic;
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
			light.theme = BASE_THEME;
			dark.theme = BASE_THEME + "-dark";

			homogeneous = true;
			add(light);
			add(dark);

			bind_property("previewing", _export, "file");

			shuffle();
		}

		public void shuffle () {
			var samples_names = random_selection(colours, 6);
			var samples = new File[6];

			for (var j = 0; j < 6; j++) {
				samples[j] = File.new_for_uri("resource:/" + RES_PATH + samples_names[j]);
			}

			light.load_samples(samples);
			dark.load_samples(samples);
		}

		public Gdk.Pixbuf screenshot () {
			var w = get_allocated_width();
			var h = get_allocated_height();
			var surface = new Cairo.ImageSurface (ARGB32, w, h);
			var context = new Cairo.Context (surface);

			draw(context);

			Gdk.Pixbuf logo;
			try {
				logo = new Gdk.Pixbuf.from_resource_at_scale ("/org/gnome/IconPreview/badge.svg", 32, -1, true);
			} catch (Error e) {
				critical (e.message);
				logo = new Gdk.Pixbuf (RGB, false, 1, 2, 2);
			}
			var layout = create_pango_layout (_("Icon Preview"));

			var padding = 8;

			var img_height = logo.get_height();
			var img_width = logo.get_width();
			Pango.Rectangle txt_extents;

			layout.get_pixel_extents(null, out txt_extents);

			var img_x = 0;
			var txt_x = img_width + padding;
			if (get_direction () == RTL) {
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

		private File? split_svg(Rsvg.Handle svg, string id) {
			if (svg.has_sub(id)) {
				FileIOStream stream;
				var temp_file = File.new_tmp("XXXXXX-" + id.substring(1, -1) +".svg", out stream);
				Rsvg.Rectangle size;
				Rsvg.Rectangle viewport = { 0.0, 0.0, svg.width, svg.height };
				svg.get_geometry_for_element(id, viewport, null, out size);
				var surface = new Cairo.SvgSurface(temp_file.get_path(), 128, 128);
				var cr = new Cairo.Context(surface);
				cr.scale(128/size.width, 128/size.height);
				cr.translate(-size.x, -size.y);
				svg.render_cairo(cr);
				return temp_file;
			}
			return null;
		}
	}
}
