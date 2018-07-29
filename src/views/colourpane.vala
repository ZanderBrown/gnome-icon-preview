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
				label.label = Path.get_basename(IconTheme.get_default().lookup_by_gicon(icon, 128, FORCE_SVG).get_filename());
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

		List<string> colours;
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
			// Not sure i should be hardcoding this
			foreach (var icon in IconTheme.get_default().list_icons("Applications")) {
				if (!icon.has_suffix("symbolic")) {
					colours.append(icon);
				}
			}

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

			shuffle();
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

		public void shuffle () {
			// Do this a two sepeperate idle callbacks
			// to avoid compleatly freezing the app
			Idle.add(() => {
				grid.foreach(image => (image as DemoIcon).icon = random());
				// Unfortunatly we have just randomised
				// The icon in grid we actually care about
				icon = icon;
				return Source.REMOVE;
			});
		}

		private Icon random () {
			var pos = Random.int_range(0, (int32) colours.length());
			return new ThemedIcon(colours.nth_data(pos));
		}
	}
}

