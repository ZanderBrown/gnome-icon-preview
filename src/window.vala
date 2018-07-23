using Gtk;

namespace IconPreview {
	[GtkTemplate (ui = "/org/gnome/IconPreview/window.ui")]
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

		FileMonitor monitor = null;

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

		public Application app {
			construct {
				application = value;
			}
		}

		public Window (Application app) {
			Object(app: app);
		}

		construct {
			var a = new Pane();
			var b = new Pane();
			b.theme = "Adwaita-dark";
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
