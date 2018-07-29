using Gtk;

namespace IconPreview {
	public class DemoIcon : Box {
		private Image image = new Image();
		private Label label = new Label(null);

		public Icon icon { get; set; }

		construct {
			orientation = VERTICAL;
			spacing = 10;

			image.pixel_size = 128;

			label.ellipsize = START;
			label.max_width_chars = 30;

			bind_property("icon", image, "gicon");
			notify["icon"].connect(() => {
				var basename = Path.get_basename(IconTheme.get_default().lookup_by_gicon(icon, 128, FORCE_SVG).get_filename());
				label.label = basename;
				label.tooltip_text = basename;
			});

			pack_start(image);
			pack_end(label);
		}
	}

	[GtkTemplate (ui = "/org/gnome/IconPreview/colourpane.ui")]
	public class ColourPane : Box {
		[GtkChild]
		Grid sizes;

		[GtkChild]
		Grid grid;

		List<Image> icons;
		DemoIcon demo_icon;
		CssProvider provider = null;

		private Icon _icon = new ThemedIcon("start-here-symbolic");
		public Icon icon {
			get {
				return _icon;
			}

			set {
				_icon = value;
				foreach (var icon in icons) {
					icon.gicon = _icon;
				}
				demo_icon.icon = _icon;
			}
		}

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
				provider = CssProvider.get_named(value, null);
				apply_css(this);
			}
		}

		class construct {
			set_css_name("pane");
		}

		construct {
			for (var i = 0; i < 3; i++) {
				icons.append(sizes.get_child_at(i, 0) as Image);
			}
			for (var i = 0; i < 3; i++) {
				var ico = new DemoIcon();
				grid.attach(ico, i, 0);
			}
			var ico = new DemoIcon();
			grid.attach(ico, 0, 1);
			demo_icon = new DemoIcon();
			grid.attach(demo_icon, 1, 1);
			ico = new DemoIcon();
			grid.attach(ico, 2, 1);
			grid.show_all();

			theme = theme;
		}

		// Adapted from one of the gtk demos
		private void apply_css(Widget widget) {
			var context = widget.get_style_context();
			StyleProvider existing = widget.get_data("pane-style-provider");
			if (existing != null) {
				context.remove_provider(existing);
			}
			context.add_provider(provider, uint.MAX - 10);
			widget.set_data("pane-style-provider", provider);
			if (widget is Container) {
				(widget as Container).forall(apply_css);
			}
		}

		public void load_samples (Icon[] samples) requires (samples.length == 5) {
			// Don't like how much of this is hardcoded
			for (var i = 0; i < 3; i++) {
				(grid.get_child_at(i, 0) as DemoIcon).icon = samples[i];
			}
			(grid.get_child_at(0, 1) as DemoIcon).icon = samples[3];
			(grid.get_child_at(2, 1) as DemoIcon).icon = samples[4];
		}
	}
}

