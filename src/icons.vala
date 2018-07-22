// (C) 2018 Zander Brown

using Gtk;

namespace IconPreview { 
	[GtkTemplate (ui = "/org/gnome/IconPreview/window.ui")]
	public class Window : ApplicationWindow {
		// Selectors
		[GtkChild]
		ComboBox mode;

		[GtkChild]
		ColorButton colour;
		[GtkChild]
		Entry iconname;
		[GtkChild]
		FileChooserButton iconfile;

		// Dummy Areas
		[GtkChild]
		ToolButton dummy1;

		[GtkChild]
		ToolButton dummy2;

		[GtkChild]
		ToolButton dummy3;

		[GtkChild]
		Button dummy4;

		[GtkChild]
		Button dummy5;

		// Demo Areas
		[GtkChild]
		Button button;

		[GtkChild]
		ToolButton toolbar;

		// Inspector View
		[GtkChild]
		Image viewer;

		[GtkChild]
		SpinButton size;

		// Available symbolic icons
		List<string> symbolics;
		// Used to set icon colour
		CssProvider provider = new CssProvider();

		// Window menu
		[GtkChild]
		MenuButton winmenu;

		const GLib.ActionEntry[] entries = {
			{ "menu",  open_menu },
			{ "about", about },
			{ "quit",  quit  }
		};

		public File file {
			set {
				try {
					iconfile.set_file(value);
					mode.active_id = "file";
					update_iconfile();
				} catch (Error e) {
					critical("Failed to set file");
				}
			}
		}

		public Application app {
			construct {
				application = value;
			}
		}
		
		public Window (Application app) {
			Object(app: app);
		}

		construct {
			update_iconname();
			update_size();

			colour.rgba = get_style_context().get_color(NORMAL);
			StyleContext.add_provider_for_screen (Gdk.Screen.get_default (), provider, STYLE_PROVIDER_PRIORITY_USER);

			var icons = new Gtk.ListStore (1, typeof (string));
			TreeIter iter;
			foreach (var icon in IconTheme.get_default().list_icons(null)) {
				icons.append (out iter);
				icons.set (iter, 0, icon);
				if (icon.has_suffix("symbolic")) {
					symbolics.append(icon);
				}
			}

			iconname.completion = new EntryCompletion ();
			iconname.completion.model = icons;
			iconname.completion.text_column = 0;

			var modes = new Gtk.ListStore (2, typeof (string), typeof (string));
			modes.append (out iter);
			modes.set (iter, 0, "Name", 1, "name");
			modes.append (out iter);
			modes.set (iter, 0, "File", 1, "file");
			mode.model = modes;

			var renderer = new CellRendererText ();
			mode.pack_start (renderer, true);
			mode.add_attribute (renderer, "text", 0);
			mode.active = 0;

			dummy1.icon_name = pick_symbolic();
			dummy2.icon_name = pick_symbolic();
			dummy3.icon_name = pick_symbolic();
			dummy4.image = new Image.from_icon_name(pick_symbolic(), BUTTON);
			dummy5.image = new Image.from_icon_name(pick_symbolic(), BUTTON);

			winmenu.menu_model = application.get_menu_by_id("win-menu");

			add_action_entries(entries, this);
		}

		private void open_menu () {
			winmenu.clicked();
		}

		private void about () {
			var authors = new string[] {"Zander Brown"};
			show_about_dialog (this,
				program_name: "Icon Preview",
				version: "%s@%s".printf(PACKAGE_VERSION, COMMIT_ID),
				copyright: "Copyright © 2018 Zander Brown",
				license_type: License.GPL_3_0,
				authors: authors,
				website: "https://gitlab.gnome.org/ZanderBrown/icon-tool/",
				website_label: "Repository");
		}

		private void quit () {
			application.quit();
		}

		private string pick_symbolic () {
			//return symbolics.nth_data(Random.int_range(0, symbolics.length));
			return symbolics.nth_data(1);
		}

		private void update_icon (Icon icon) {
			viewer.gicon = icon;
			toolbar.icon_widget = new Image.from_gicon(icon, BUTTON);
			button.image = new Image.from_gicon(icon, BUTTON);
		}

		[GtkCallback]
		private void mode_changed() {
			switch (mode.active_id) {
				case "file":
					iconfile.visible = true;
					iconname.visible = false;
					break;
				case "name":
					iconfile.visible = false;
					iconname.visible = true;
					break;
				default:
					critical("Something bad happened when changing icon mode");
					break;
			}
		}

		[GtkCallback]
		private void update_iconname () {
			update_icon(new ThemedIcon(iconname.text));
		}

		[GtkCallback]
		private void update_iconfile () {
			update_icon(new FileIcon(iconfile.get_file()));
		}

		[GtkCallback]
		private void update_size () {
			viewer.pixel_size = (int) size.value;
		}

		[GtkCallback]
		private void update_colour () {
			var tmpl = ".preview-area image { color: %s; }";
			try {
				provider.load_from_data(tmpl.printf(colour.rgba.to_string()));
			} catch (Error e) {
				message("Couldn't set colour: %s", e.message);
			}
		}
	}
}
