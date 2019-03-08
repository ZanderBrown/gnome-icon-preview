using Gtk;

namespace IconPreview {
	class PaletteButton : Button {
		private CssProvider provider = new CssProvider();
		private string _hex = "";

		public string hex {
			get {
				return _hex;
			}

			set {
				_hex = value;
				tooltip_text = value;
				try {
					provider.load_from_data("* { background: %s; }".printf(value));
				} catch (Error e) {
					warning("Can't set colour! %s", e.message);
				}
			}
		}

		construct {
			visible = true;

			width_request = 32;
			height_request = 32;

			// We really want to set the background
			get_style_context().add_provider(provider, STYLE_PROVIDER_PRIORITY_USER);
		}

		public PaletteButton (string hex) {
			Object(hex: hex);
		}

		public override void clicked () {
			Clipboard.get_default(get_display()).set_text(hex, -1);
		}
	}

	[GtkTemplate (ui = "/org/gnome/IconPreview/palette.ui")]
	class Palette : ApplicationWindow {
		[GtkChild]
		private Box blue;

		[GtkChild]
		private Box green;

		[GtkChild]
		private Box yellow;

		[GtkChild]
		private Box orange;

		[GtkChild]
		private Box red;

		[GtkChild]
		private Box purple;

		[GtkChild]
		private Box brown;

		[GtkChild]
		private Box light;

		[GtkChild]
		private Box dark;

		[GtkChild]
		private MenuButton menu;

		const GLib.ActionEntry[] entries = {
			{ "about", about },
		};

		// A hack to get construction to work properly
		public Application app {
			construct {
				application = value;
			}

			private get {
				return application as Application;
			}
		}

		construct {
			// Bind the actions
			add_action_entries(entries, this);

			var file = File.new_for_uri("resource:///org/gnome/IconPreview/palette.gpl");
			var regex = /(?<r>\d+)\s+(?<g>\d+)\s+(?<b>\d+)\s+(?<name>\w+)/;
			try {
				var dis = new DataInputStream (file.read ());
				string l;
				while ((l = dis.read_line (null)) != null) {
					var line = l.strip();
					var c = line.get_char(0);
					if (c.isdigit()) {
						MatchInfo info;
						if(regex.match(line, 0, out info)) {
							var r = int.parse(info.fetch_named("r"));
							var g = int.parse(info.fetch_named("g"));
							var b = int.parse(info.fetch_named("b"));
							var name = info.fetch_named("name");

							var hex = "#%02x%02x%02x".printf(r, g, b);
							var btn = new PaletteButton(hex);

							switch (name) {
								case "Blue":
								blue.add(btn);
								break;
								case "Green":
								green.add(btn);
								break;
								case "Yellow":
								yellow.add(btn);
								break;
								case "Orange":
								orange.add(btn);
								break;
								case "Red":
								red.add(btn);
								break;
								case "Purple":
								purple.add(btn);
								break;
								case "Brown":
								brown.add(btn);
								break;
								case "Light":
								light.add(btn);
								break;
								case "Dark":
								dark.add(btn);
								break;
								default:
								warning("Unknown group %s", name);
								break;
							}
						}
					}
				}
			} catch (Error e) {
				error ("%s", e.message);
			}

			menu.menu_model = application.get_menu_by_id("win-palette-menu");
		}

		public Palette (Application app) {
			Object(app: app);
		}

		// Show the about dialog, triggered by win.about
		private void about () {
			app.about(this);
		}
	}
}

