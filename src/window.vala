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
			{ "fullscreen", toggle_fullscreen },
			{ "about", about },
			{ "quit",  quit  }
		};

		bool is_fullscreen = false;

		FileMonitor monitor = null;
		Recents recents = new Recents();

		private File _file;
		public File file {
			set {
				try {
					// Hopefully this doesn't render the SVG?
					var svg = new Rsvg.Handle.from_gfile_sync(value, FLAGS_NONE);
					// Colour (App) icons must be 128 by 128
					if (svg.height == 128 && svg.width == 128) {
						mode = COLOUR;
					// Whereas symbolics are 16 by 16
					} else if (svg.height == 16 && svg.width == 16) {
						mode = SYMBOLIC;
					// And anything else is unsupported
					} else {
						// We are very specific about what we like
						_file = null;
						_load_failed();
						// Give up now
						return;
					}
				} catch (Error e) {
					// rsvg didn't like it (not an SVG?)
					critical("Failed to load %s: %s", value.get_basename(), e.message);
					_file = null;
					_load_failed();
					return;
				}
				// Tell the recents popover we opened this
				recents.open_file(value);
				try {
					// If we are already monitoring an open file
					if (monitor != null) {
						// Stop doing that
						monitor.cancel();
					}
					// Watch for updates
					monitor = value.monitor_file(NONE, null);
					monitor.changed.connect(file_updated);
				} catch (Error e) {
					// Failed to watch the file
					critical("Unable to watch icon: %s", e.message);
				}
				_file = value;
				// Actually display the thing
				refresh();
			}

			get {
				return _file;
			}
		}

		public Mode mode { get; set; default = INITIAL; }

		// A timeout id
		private uint pulser = 0;
		// Controls the windows indeterminate progress bar
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

		// A hack to get construction to work properly
		public Application app {
			construct {
				application = value;
			}
		}

		public Window (Application app) {
			Object(app: app);
		}

		construct {
			// Bind the actions
			add_action_entries(entries, this);

			// Setup the initial state
			var inital = new InitialState();
			inital.show();
			content.add(inital);
			content.visible_child = inital;

			// Connect the recent button and recent popover
			recent.popover = recents;
			// Load files selected in the popover
			recents.open.connect(recent => file = recent);

			// Listen for changes to the mode
			notify["mode"].connect(mode_changed);
			// Manually trigger a change
			mode_changed();

			// For some reason MenuButton doesn't have a menu_id property
			menu.menu_model = application.get_menu_by_id("win-menu");
		}

		private void _load_failed () {
			var dlg = new MessageDialog(this, MODAL, WARNING, CANCEL, "This file is defective");
			dlg.secondary_text = "Please start from a template to ensure that your file will work as a GNOME icon";
			dlg.response.connect(() => dlg.destroy());
			dlg.show();
		}

		private void mode_changed () {
			exportbtn.visible = mode != INITIAL;
			switch (mode) {
				case INITIAL:
					title = "Icon Preview";
					(lookup_action("refresh") as SimpleAction).set_enabled(false);
					(lookup_action("shuffle") as SimpleAction).set_enabled(false);
					(lookup_action("export") as SimpleAction).set_enabled(false);
					break;
				case SYMBOLIC:
					_mode_changed(new Symbolic());
					break;
				case COLOUR:
					_mode_changed(new Colour());
					break;
			}
		}

		private void _mode_changed (Previewer view) {
			var old = content.visible_child;
			view.show();
			content.add(view);
			content.visible_child = view;
			var pop = new Popover(exportbtn);
			pop.add(view.exporter);
			exportbtn.popover = pop;
			if (old is InitialState) {
				// We have an open file now
				(lookup_action("refresh") as SimpleAction).set_enabled(true);
				(lookup_action("shuffle") as SimpleAction).set_enabled(true);
				(lookup_action("export") as SimpleAction).set_enabled(true);
			} else {
				// Effectivly close the old previewer
				old.destroy();
			}
		}

		private void open () {
			var dlg = new FileChooserNative("Select Icon", this, OPEN, null, null);
			var filter = new Gtk.FileFilter ();
			filter.set_filter_name ("Icons");
			filter.add_pattern ("*.svg");
			filter.add_mime_type ("image/svg+xml");
			dlg.add_filter (filter);
			dlg.response.connect(res => {
				if (res == ResponseType.ACCEPT) {
					file = dlg.get_file();
				}
			});
			dlg.show();
		}

		// win.new always expects an argument
		private void new_icon (GLib.Action _act, Variant? arg) requires (arg != null) {
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

		// Open the recent popover (win.recents)
		private void open_recent () {
			recent.clicked();
		}

		// Open the export popover (win.export)
		private void open_export () {
			exportbtn.clicked();
		}

		// Become / leave fullscreen (win.fullscreen)
		private void toggle_fullscreen () {
			if (is_fullscreen) {
				is_fullscreen = false;
				unfullscreen();
			} else {
				is_fullscreen = true;
				fullscreen();
			}
		}

		// Manually reload the current icon (win.refresh)
		// Requires:
		//     The must be an open file to reload
		private void refresh () requires (file != null) {
			// Trigger a dummy changed event
			file_updated(file, null, CHANGED);
		}

		// Change the random comparison icons (win.shuffle)
		// Requires:
		//     The should be an open previewer to shuffle
		private void shuffle () requires (content.visible_child is Previewer) {
			(content.visible_child as Previewer).shuffle();
		}

		// The currently open file was modified
		// Requires:
		//     The source of the event shouldn't be null and a previewer has no
		//     chance of displaying null, equally there must be an active
		//     previewer to display the modified icon in
		private void file_updated (File src, File? dest, FileMonitorEvent evt) requires (src != null && content.visible_child is Previewer) {
			if (evt != CHANGED) {
				return;
			}
			(content.visible_child as Previewer).previewing = src;
			try {
				var info = src.query_info ("standard::display-name", NONE);
				title = info.get_display_name();
			} catch (Error e) {
				critical("Failed to fetch icon name: %s", e.message);
				title = "Icon Preview";
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
				version: PACKAGE_VERSION,
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
