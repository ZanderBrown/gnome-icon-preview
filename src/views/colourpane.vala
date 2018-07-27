using Gtk;

namespace IconPreview {
	[GtkTemplate (ui = "/org/gnome/IconPreview/colourpane.ui")]
	public class ColourPane : Box {
		[GtkChild]
		Grid sizes;

		[GtkChild]
		Grid grid;

		List<string> colours;
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
			}
		}

		construct {
			set_css_name("pane");

			foreach (var icon in IconTheme.get_default().list_icons(null)) {
				if (!icon.has_suffix("symbolic")) {
					colours.append(icon);
				}
			}

			for (var i = 0; i < 3; i++) {
				icons.append(sizes.get_child_at(i, 0) as Image);
			}
			icons.append(grid.get_child_at(1, 1) as Image);
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
	}
}

