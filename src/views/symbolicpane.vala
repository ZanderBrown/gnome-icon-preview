using Gtk;

namespace IconPreview {
	[GtkTemplate (ui = "/org/gnome/IconPreview/symbolicpane.ui")]
	public class SymbolicPane : Box {
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
				_icon = value;
				foreach (var icon in icons) {
					icon.gicon = _icon;
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

		class construct {
			set_css_name("pane");
		}

		construct {
			foreach (var icon in IconTheme.get_default().list_icons(null)) {
				if (icon.has_suffix("symbolic")) {
					symbolics.append(icon);
				}
			}

			for (var i = 0; i < 3; i++) {
				icons.append(sizes.get_child_at(i, 0) as Image);
			}
			icons.append(grid.get_child_at(3, 1) as Image);
			var count = 0;
			// Doesn't seem to be a way to directly access
			linked.foreach(btn => {
				if (count == 2) {
					icons.append((btn as Button).image as Image);
				}
				count++;
			});
			states.foreach(state => {
				icons.append((state as Button).image as Image);
			});

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
				grid.foreach(image => (image as Image).gicon = random());
				// Unfortunatly we have just randomised
				// The icon in grid we actually care about
				icon = icon;
				return Source.REMOVE;
			});
			Idle.add(() => {
				var count = 0;
				linked.foreach(btn => {
					if (count != 2) {
						((btn as Button).image as Image).gicon = random();
					}
					count++;
				});
				return Source.REMOVE;
			});
		}

		private Icon random () {
			var pos = Random.int_range(0, (int32) symbolics.length());
			return new ThemedIcon(symbolics.nth_data(pos));
		}
	}
}

