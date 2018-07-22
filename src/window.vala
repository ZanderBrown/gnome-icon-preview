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

		private Icon random () {
			var pos = Random.int_range(0, (int32) symbolics.length());
			return new ThemedIcon(symbolics.nth_data(pos));
		}
	}

	[GtkTemplate (ui = "/org/gnome/IconPreview/newwindow.ui")]
	public class Window : ApplicationWindow {
		[GtkChild]
		Box panes;

		[GtkChild]
		MenuButton menu;

		const GLib.ActionEntry[] entries = {
			{ "open", open },
			{ "refresh", refresh },
			{ "menu",  open_menu },
			{ "about", about },
			{ "quit",  quit  }
		};

		private Icon _icon = new ThemedIcon("start-here-symbolic");
		public Icon preview_icon {
			get {
				return _icon;
			}
			set {
				panes.foreach(pane => (pane as Pane).icon = value);
			}
		}

		private File _file;
		public File file {
			set {
				if (monitor != null) {
					monitor.cancel();
				}
				try {
					monitor = value.monitor_file(NONE, null);
					monitor.changed.connect(file_updated);
				} catch (Error e) {
					critical("Unable to watch icon: %s", e.message);
				}
				file_updated(value, null, CHANGED);
				_file = value;
			}
			get {
				return _file;
			}
		}

		FileMonitor monitor = null;

		public Window (Application app) {
			Object(application: app);
		}

		construct {
			var a = new Pane();
			var b = new Pane();
			panes.pack_start(a);
			panes.pack_end(b);

			menu.menu_model = application.get_menu_by_id("win-menu");
			add_action_entries(entries, this);
		}

		private void open () {
			var dlg = new FileChooserNative("Select Icon", this, OPEN, null, null);
			dlg.response.connect(res => {
				if (res == ResponseType.ACCEPT) {
					file = dlg.get_file();
				}
			});
			dlg.show();
		}

		private void refresh () {
			file_updated(file, null, CHANGED);
		}

		private void file_updated (File src, File? dest, FileMonitorEvent evt) {
			if (evt == CHANGED) {
				preview_icon = new FileIcon(src);
				try {
					var info = src.query_info ("standard::display-name", NONE);
					title = info.get_display_name();
				} catch (Error e) {
					critical("Failed to fetch icon name: %s", e.message);
					title = "Icon Preview";
				}
			}
		}

		// Wrapper for win.menu
		private void open_menu () {
			menu.clicked();
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
	}
}
