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

		[GtkChild]
		Grid cloud;

		// Demo Areas
		[GtkChild]
		Button button;

		[GtkChild]
		Button button2;

		[GtkChild]
		Button button3;

		[GtkChild]
		Button button4;

		[GtkChild]
		Button button5;

		[GtkChild]
		Button button6;

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
		// Holds all our dummy buttons (so we can iterate them)
		List<Button> buttons;

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

			buttons.append(button);
			buttons.append(button2);
			buttons.append(button3);
			buttons.append(button4);
			buttons.append(button5);
			buttons.append(button6);

			dummy1.icon_name = pick_symbolic();
			dummy2.icon_name = pick_symbolic();
			dummy3.icon_name = pick_symbolic();
			dummy4.image = new Image.from_icon_name(pick_symbolic(), BUTTON);
			dummy5.image = new Image.from_icon_name(pick_symbolic(), BUTTON);

			cloud.foreach(image => {
				(image as Image).icon_name = pick_symbolic();
			});

			update_iconname();
			update_size();

			winmenu.menu_model = application.get_menu_by_id("win-menu");

			add_action_entries(entries, this);
		}

		// Wrapper for win.menu
		private void open_menu () {
			winmenu.clicked();
		}

		// Show the about dialog, triggered by win.about
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

		// Wrapper for win.quit
		private void quit () {
			application.quit();
		}

		// Choose a random symbolic icon
		private string pick_symbolic () {
			return symbolics.nth_data(Random.int_range(0, (int32) symbolics.length()));
		}

		// Update everywhere the selected icon is displayed
		private void update_icon (Icon icon) {
			viewer.gicon = icon;
			toolbar.icon_widget = new Image.from_gicon(icon, BUTTON);
			toolbar.icon_widget.show();
			foreach (var button in buttons) {
				button.image = new Image.from_gicon(icon, BUTTON);
				button.image.show();
			}
			(cloud.get_child_at(2, 2) as Image).gicon = icon;
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
