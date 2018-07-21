// (C) 2018 Zander Brown

using Gtk;

namespace IconPreview { 
	enum IconMode {
		NAME,
		FILE
	}

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

		List<string> symbolics;
		CssProvider provider = new CssProvider();
		
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

			var modes = new Gtk.ListStore (2, typeof (string), typeof (IconMode));
			modes.append (out iter);
			modes.set (iter, 0, "Name", 1, IconMode.NAME);
			modes.append (out iter);
			modes.set (iter, 0, "File", 1, IconMode.FILE);
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
			Value id;
			TreeIter iter;
			mode.get_active_iter (out iter);
			mode.model.get_value (iter, 1, out id);
			switch (id.get_enum()) {
				case IconPreview.IconMode.FILE:
					iconfile.visible = true;
					iconname.visible = false;
					break;
				case IconPreview.IconMode.NAME:
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
