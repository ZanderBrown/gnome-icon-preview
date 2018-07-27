using Gtk;

namespace IconPreview {
	[GtkTemplate (ui = "/org/gnome/IconPreview/window.ui")]
	public class Window : ApplicationWindow {
		[GtkChild]
		ProgressBar progress;

		[GtkChild]
		Stack content;

		[GtkChild]
		MenuButton recent;

		[GtkChild]
		Button refreshbtn;

		[GtkChild]
		MenuButton menu;

		[GtkChild]
		MenuButton exportbtn;


		const GLib.ActionEntry[] entries = {
			{ "open", open },
			{ "new-icon", new_icon, "s" },
			{ "recents", open_recent },
			{ "refresh", refresh },
			{ "shuffle", shuffle },
			{ "menu",  open_menu },
			{ "export", open_export },
			{ "about", about },
			{ "quit",  quit  }
		};

		FileMonitor monitor = null;
		Recents recents = new Recents();
		Export export_pop = new Export();

		private File _file;
		public File file {
			set {
				try {
					// Hopefully this doesn't render the SVG?
					var svg = new Rsvg.Handle.from_gfile_sync(value, FLAGS_NONE);
					if (svg.height == 128 && svg.width == 128) {
						mode = COLOUR;
					} else if (svg.height == 16 && svg.width == 16) {
						mode = SYMBOLIC;
					} else {
						// We are very specific about what we like
						mode = INITIAL;
						_file = null;
						return;
						// TODO report bad icon
					}
				} catch (Error e) {
					critical("Failed to load %s: %s", value.get_basename(), e.message);
					mode = INITIAL;
					_file = null;
					return;
					// TODO report bad icon
				}
				recents.open_file(value);
				try {
					if (monitor != null) {
						monitor.cancel();
					}
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

		public Mode mode { get; set; default = INITIAL; }

		private uint pulser = 0;
		public bool pulsing {
			set {
				if (value) {
					pulser = Timeout.add(500, () => {
						progress.pulse();
						return Source.CONTINUE;
					});
				} else {
					Source.remove(pulser);
				}
			}
			get {
				return pulser != 0;
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
			var inital = new InitialState();
			content.add(inital);
			content.visible_child = inital;

			//var symbolics = new Symbolic();
			//content.add(symbolics);
			//symbolics.show();
			//content.visible_child = symbolics;

			recent.popover = recents;
			recents.open.connect(recent => file = recent);

			exportbtn.popover = export_pop;
			bind_property("mode", export_pop, "mode");

			notify["mode"].connect(mode_changed);
			mode_changed();

			menu.menu_model = application.get_menu_by_id("win-menu");
			add_action_entries(entries, this);
		}

		private void mode_changed () {
			message("Switched to %s", mode.to_string());
			refreshbtn.visible = exportbtn.visible = mode != INITIAL;
			switch (mode) {
				case INITIAL:
					if (!(content.visible_child is InitialState)) {
						content.visible_child.destroy();
						message("Destroyed");
					}
					break;
				case SYMBOLIC:
					var view = new Symbolic();
					view.show();
					content.add_named(view, "symbolic");
					content.visible_child_name = "symbolic";
					break;
				case COLOUR:
					var view = new Colour();
					view.show();
					content.add_named(view, "colour");
					content.visible_child = view;
					view.show();
					break;
			}
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

		private void new_icon (GLib.Action _act, Variant? arg) {
			if (arg == null) {
				critical("Expected argument for win.new");
				return;
			}
			var dlg = new FileChooserNative("New Icon", this, SAVE, "_Save", "_Cancel");
			dlg.response.connect(res => {
				if (res == ResponseType.ACCEPT) {
					switch (arg.get_string()) {
						case "symbolic":
							_new_icon.begin(dlg.get_file(), "symbolic.svg");
							break;
						case "colour":
							_new_icon.begin(dlg.get_file(), "colour.svg");
							break;
						default:
							critical("Bad argument for win.new");
							break;
					}
				}
			});
			dlg.show();
		}


		private async void _new_icon (File dest, string src) {
			progress.visible = true;
			pulsing = true;
			var from = File.new_for_uri("resource:///org/gnome/IconPreview/templates/" + src);
			try {
				yield from.copy_async (dest, NONE);
				message("Copied %s -> %s", from.get_uri(), dest.get_uri());
				/*var context = get_display().get_app_launch_context();
				context.set_screen (screen);
				context.set_timestamp (Gdk.CURRENT_TIME);
				context.launched.connect(with => message("Opened with %s", with.get_display_name()));
				context.launch_failed.connect(() => critical("Failed to launch template"));
				message("Open: %s", dest.get_uri());*/
				//yield AppInfo.launch_default_for_uri_async(dest.get_uri(), context);
				//yield AppInfo.launch_default_for_uri_async("https://example.com", context);
				message("Launched? %s", AppInfo.launch_default_for_uri(dest.get_uri(), null).to_string());
				// TODO: Why doesn't this work? ^^^^^^^^^^^^^^^^^^^^^^
			} catch (Error e) {
				critical ("Error: %s", e.message);
			}
			file = dest;
			pulsing = false;
			progress.visible = false;
		}

		private void open_recent () {
			recent.clicked();
		}

		private void open_export () {
			export_pop.popup();
		}

		private void refresh () {
			file_updated(file, null, CHANGED);
		}

		private void shuffle () {
			if (content.visible_child is Previewer) {
				(content.visible_child as Previewer).shuffle();
			}
		}

		private void file_updated (File src, File? dest, FileMonitorEvent evt) {
			if (evt == CHANGED) {
				if (content.visible_child is Previewer) {
					(content.visible_child as Previewer).previewing = src;
				}
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
			var artists = new string[] {"Tobias Bernard"};
			show_about_dialog (this,
				program_name: "Icon Preview",
				logo_icon_name: "org.gnome.IconPreview",
				version: "%s@%s".printf(PACKAGE_VERSION, COMMIT_ID),
				copyright: "Copyright © 2018 Zander Brown",
				license_type: License.GPL_3_0,
				authors: authors,
				artists: artists,
				website: "https://gitlab.gnome.org/ZanderBrown/icon-tool/",
				website_label: "Repository");
		}

		// Wrapper for win.quit
		private void quit () {
			application.quit();
		}
	}
}
