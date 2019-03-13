using Gtk;

namespace IconPreview {
	public class DemoIcon : Box {
		private Image image = new Image();
		private Label label = new Label(null);

		public Icon icon { get; set; }
		public int size { get; construct set; default = 96; }

		class construct {
			set_css_name("demo-icon");
		}

		construct {
			orientation = VERTICAL;
			spacing = 5;
			expand = false;
			valign = CENTER;

			label.ellipsize = MIDDLE;
			label.max_width_chars = 30;

			bind_property("icon", image, "gicon");
			bind_property("size", image, "pixel_size");
			notify["icon"].connect(() => {
				var basename = Path.get_basename(IconTheme.get_default().lookup_by_gicon(icon, 96, FORCE_SVG).get_filename());
				var parts = basename.split(".");
				label.label = parts[parts.length - 2];
				label.tooltip_text = basename;
			});

			pack_start(image);
			pack_end(label);
		}

		public DemoIcon (int size) {
			Object (size: size);
		}
	}

	[GtkTemplate (ui = "/org/gnome/IconPreview/colourpane.ui")]
	public class ColourPane : Box {
		[GtkChild]
		Grid sizes;

		[GtkChild]
		Box grid;

		[GtkChild]
		Box small;

		CssProvider provider = null;
		List<DemoIcon> randoms;

		public Icon icon { get; set; default =  new ThemedIcon("start-here-symbolic");}

		private string _theme = "Adwaita";
		public string theme {
			get {
				return _theme;
			}

			set {
				var context = get_style_context();
				context.remove_class("theme-" + _theme);
				_theme = value;
				context.add_class("theme-" + _theme);
				var parts = _theme.split("-");
				if (parts.length > 1 && parts[1] == "dark") {
					provider = CssProvider.get_named(parts[0], "dark");
				} else {
					provider = CssProvider.get_named(_theme, null);
				}
				apply_css(this, provider);
			}
		}

		class construct {
			set_css_name("pane");
		}

		construct {
			DemoIcon ico;

			for (var i = 0; i < 3; i++) {
				bind_property("icon", sizes.get_child_at(i, 0), "gicon");
			}

			/* 64px                            */
			for (var i = 0; i < 2; i++) {
				ico = new DemoIcon(64);
				small.add(ico);
				randoms.append(ico);
			}

			ico = new DemoIcon(64);
			bind_property("icon", ico, "icon");
			small.add(ico);

			for (var i = 3; i < 5; i++) {
				ico = new DemoIcon(64);
				small.add(ico);
				randoms.append(ico);
			}

			small.show_all();
			/* 64px                            */

			/* 96px                            */
			ico = new DemoIcon(96);
			grid.add(ico);
			randoms.append(ico);

			ico = new DemoIcon(96);
			bind_property("icon", ico, "icon");
			grid.add(ico);

			ico = new DemoIcon(96);
			grid.add(ico);
			randoms.append(ico);
			grid.show_all();
			/* 96px                            */

			theme = theme;
		}

		public void load_samples (Icon[] samples) requires (samples.length == randoms.length()) {
			// Don't like how much of this is hardcoded
			var idx = 0;
			foreach (var sample in randoms) {
				sample.icon = samples[idx];
				idx++;
			}
		}
	}
}

