using Gtk;

namespace IconPreview {
	public class DemoIcon : Box {
		private Image image = new Image();
		private Label label = new Label(null);
		public File ?file { get; construct set; }
		public string ?name { get; set; }

		public int size { get; construct set; default = 96; }
		class construct {
			set_css_name("demo-icon");
		}

		construct {
			orientation = VERTICAL;
			spacing = 5;
			expand = false;
			valign = CENTER;

			label.ellipsize = END;
			label.max_width_chars = 30;

			bind_property("size", image, "pixel_size");
			notify["file"].connect((s, p) => {
				if (name != null) {
					var parts = name.split(".");
					label.label = parts[parts.length - 2];
					label.tooltip_text = name;
					image.gicon = new FileIcon(file);
				}
			});

			pack_start(image);
			pack_end(label);
		}

		public DemoIcon (int size) {
			Object (size: size);
		}
	}

	[GtkTemplate (ui = "/org/gnome/design/IconPreview/colourpane.ui")]
	public class ColourPane : Box {
		[GtkChild]
		Grid sizes;

		[GtkChild]
		Box grid;

		[GtkChild]
		Box small;

		CssProvider provider = null;
		List<DemoIcon> randoms;

		public File hicolor { get; set; }
		public File ?symbolic { get; set; }

		public string name { get; set; }

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

			notify["hicolor"].connect(() => {
				if (symbolic == null) {
					//hide the symbolic icon in the preview
					sizes.get_child_at(0, 0).hide();
					sizes.get_child_at(0, 1).hide();
				}
				FileIcon icon = new FileIcon(hicolor);
				// Three different sizes {32, 64, 128};
				for (var i = 0; i < 3; i++) {
					var image = sizes.get_child_at(i + 1, 0) as Image;
					image.set_from_gicon(icon, BUTTON);
				}
			});

			notify["symbolic"].connect(() => {
				if (symbolic != null) {
					sizes.get_child_at(0, 1).show();
					var image = sizes.get_child_at(0, 0) as Image;
					image.show();
					FileIcon icon = new FileIcon(symbolic);
					image.set_from_gicon(icon, BUTTON);
				} else {
					//hide the symbolic icon in the preview
					sizes.get_child_at(0, 0).hide();
					sizes.get_child_at(0, 1).hide();
				}
			});

			/* 64px                            */
			for (var i = 0; i < 2; i++) {
				ico = new DemoIcon(64);
				small.add(ico);
				randoms.append(ico);
			}

			/* add 64x64 users icon preivew */
			ico = new DemoIcon(64);
			bind_property("hicolor", ico, "file");
			bind_property("name", ico, "name");
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

			/* add 96x96 users icon preivew */
			ico = new DemoIcon(96);
			bind_property("hicolor", ico, "file");
			bind_property("name", ico, "name");
			grid.add(ico);

			ico = new DemoIcon(96);
			grid.add(ico);
			randoms.append(ico);
			grid.show_all();
			/* 96px                            */

			theme = theme;
		}

		public void load_samples (File[] samples) requires (samples.length == randoms.length()) {
			// Don't like how much of this is hardcoded
			var idx = 0;
			foreach (var sample in randoms) {
				sample.name = samples[idx].get_basename();
				sample.file = samples[idx];
				idx++;
			}
		}
	}
}

