using Gtk;

namespace IconPreview {
	public class Colour : Box, Previewer {
		const string RES_PATH = "/org/gnome/design/IconPreview/icons/";
		const string BASE_THEME = "Adwaita";
		static string[] colours;

		private ColourPane light = new ColourPane();
		private ColourPane dark = new ColourPane();
		private Exporter exporter;

		private File _icon;
		public File previewing {
			get {
				return _icon;
			}
			set {
				_icon = value;
				var svg = new Rsvg.Handle.from_gfile_sync(_icon, FLAGS_NONE);
				var hicolor = create_tmp_file ("#hicolor");
				render_by_id(svg, "#hicolor", hicolor, 128);

				var nightly = create_tmp_file ("#nightly");
				var nightly_surface = render_by_id(svg, "#hicolor", nightly, 128);
				make_nightly(nightly_surface, 128);

				var symbolic = create_tmp_file ("#symbolic");
        render_by_id(svg, "#symbolic", symbolic, 16);

				light.name = dark.name = _icon.get_basename();

				if (hicolor != null) {
					light.hicolor = dark.hicolor = hicolor;
				} else {
					light.hicolor = dark.hicolor = _icon;
				}

				exporter.update_regolar(light.hicolor);
				exporter.update_nightly(nightly);
				exporter.update_symbolic(symbolic);
				exporter.name = light.name;
				light.symbolic = dark.symbolic = symbolic;
			}
		}

		public Colour(Exporter e) {
			exporter = e;
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
			var content_h = get_allocated_height();

			Gdk.Pixbuf logo;
			try {
				logo = new Gdk.Pixbuf.from_resource_at_scale ("/org/gnome/design/IconPreview/badge.svg", 32, -1, true);
			} catch (Error e) {
				critical (e.message);
				logo = new Gdk.Pixbuf (RGB, false, 1, 2, 2);
			}
			var layout = create_pango_layout (_("Icon Preview"));

			var padding = 6;

			var img_height = logo.get_height();
			var img_width = logo.get_width();
			Pango.Rectangle txt_extents;

			layout.get_pixel_extents(null, out txt_extents);

			var bottom_bar = int.max(img_height, txt_extents.height) + (padding * 2);

			var surface = new Cairo.ImageSurface (ARGB32, w, content_h + bottom_bar);
			var context = new Cairo.Context (surface);

			draw(context);

			var img_x = 0;
			var txt_x = img_width + padding;
			if (get_direction () == RTL) {
				img_x = txt_extents.width + padding;
				txt_x = 0;
			}

			var img_y = content_h;
			var txt_y = 0;
			if (txt_extents.height < img_height) {
				txt_y = content_h + (img_height - txt_extents.height) / 2;
			} else {
				img_y = content_h + (txt_extents.height - img_height) / 2;
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

			return Gdk.pixbuf_get_from_surface (surface, 0, 0, w, content_h + bottom_bar);
		}

		private Cairo.Surface? render_by_id (Rsvg.Handle svg, string id, File file, int output_size) {
			if (svg.has_sub(id)) {
				Rsvg.Rectangle size;
				Rsvg.Rectangle viewport = { 0.0, 0.0, svg.width, svg.height };
				svg.get_geometry_for_layer (id, viewport, null, out size);
				var surface = new Cairo.SvgSurface(file.get_path(), output_size, output_size);
				surface.set_document_unit (Cairo.SvgUnit.PX);
				var cr = new Cairo.Context(surface);
				cr.scale(output_size/size.width, output_size/size.height);
				cr.translate(-size.x, -size.y);
				svg.render_cairo(cr);
				return surface;
			}
			return null;
		}

    private File create_tmp_file (string id) {
      FileIOStream stream;
      return File.new_tmp("XXXXXX-" + id.substring(1, -1) +".svg", out stream);
    }

    // This adds the nightly stripes to a Cairo.Surface
    private void make_nightly (Cairo.Surface? hicolor, int output_size) {
      if (hicolor != null) {
        debug ("Add nightly stripes");
        var cr = new Cairo.Context(hicolor);
        cr.set_source_surface(get_overlay(), 0.0, 0.0);
        var mask = new Cairo.Surface.similar (hicolor, Cairo.Content.ALPHA, output_size, output_size);
        var cr_mask = new Cairo.Context (mask);
        cr_mask.set_source_surface(hicolor, 0.0, 0.0);
        cr_mask.paint();
        cr.mask_surface(mask, 0.0, 0.0);
      }
    }

    private Cairo.Surface get_overlay() {
      var stripes = File.new_for_uri ("resource:///org/gnome/design/IconPreview/templates/stripes.svg");
      var handle = new Rsvg.Handle.from_gfile_sync (stripes, FLAGS_NONE);
      FileIOStream stream;
      var temp_file = File.new_tmp("XXXXXX-strips.svg", out stream);
      //FIXME: Vala doesn't allow us to use null for a new SvgSurface
      var surface = new Cairo.SvgSurface(temp_file.get_path(), 128, 128);
      var cr = new Cairo.Context(surface);
      handle.render_cairo(cr);
      return surface;
    }
  }
}
