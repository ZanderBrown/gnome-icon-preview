using Gtk;

namespace IconPreview {
	[GtkTemplate (ui = "/org/gnome/IconPreview/pane.ui")]
	public class Pane : Box {
		[GtkChild]
		Grid sizes;

		[GtkChild]
		Grid grid;

		[GtkChild]
		Box linked;

		[GtkChild]
		Box states;

		List<string> symbolics;
		List<Image> icons;
		CssProvider provider = null;

		private Icon _icon = new ThemedIcon("start-here-symbolic");
		public Icon icon {
			get {
				return _icon;
			}
			set {
				foreach (var icon in icons) {
					icon.gicon = value;
				}
			}
		}

		private string _theme = "Adwaita";
		public string theme {
			get {
				return _theme;
			}
			set {
				_theme = value;
				provider = CssProvider.get_named(value, null);
				apply_css(this);
				message("Load %s", value);
			}
		}

		construct {
			set_css_name("pane");

			foreach (var icon in IconTheme.get_default().list_icons(null)) {
				if (icon.has_suffix("symbolic")) {
					symbolics.append(icon);
				}
			}

			for (var i = 0; i < 3; i++) {
				icons.append(sizes.get_child_at(i, 0) as Image);
			}
			grid.foreach(image => (image as Image).gicon = random());
			icons.append(grid.get_child_at(3, 1) as Image);
			var count = 0;
			linked.foreach(btn => {
				if (count != 2) {
					(btn as Button).image = new Image.from_gicon(random(), BUTTON);
				} else {
					(btn as Button).image = new Image.from_gicon(icon, BUTTON);
					icons.append((btn as Button).image as Image);
				}
				count++;
			});
			states.foreach(state => {
				var img = new Image.from_gicon(icon, BUTTON);
				(state as Button).image = img;
				icons.append(img);
			});
		}

		// Adapted from one of the gtk demos
		private void apply_css(Widget widget) {
			var context = widget.get_style_context();
			StyleProvider existing = widget.get_data("pane-style-provider");
			if (existing != null) {
				context.remove_provider(existing);
			}
			context.add_provider(provider, uint.MAX);
			widget.set_data("pane-style-provider", provider);
			if (widget is Container) {
				(widget as Container).forall(apply_css);
			}
		}

		private Icon random () {
			var pos = Random.int_range(0, (int32) symbolics.length());
			return new ThemedIcon(symbolics.nth_data(pos));
		}
	}
}

