using Gtk;

namespace IconPreview {
	[GtkTemplate (ui = "/org/gnome/IconPreview/wizard.ui")]
	class Wizard : Dialog {
		[GtkChild]
		Label icon_type;

		[GtkChild]
		Label desc;

		[GtkChild]
		Dazzle.FileChooserEntry location;

		[GtkChild]
		Entry icon_title;

		[GtkChild]
		Spinner spin;

		[GtkChild]
		Button accept_button;

		public Mode mode { get; construct; }

		public signal void open (File file);

		construct {
			// Why is this int? Why is this not automatic?
			use_header_bar = (int) Gtk.Settings.get_default().gtk_dialogs_use_header;
			// $HOME/Projects/Icons
			location.file = File.new_for_path(Path.build_filename(Environment.get_home_dir(), "Projects", "Icons"));

			if (mode == SYMBOLIC) {
				title = "New Symbolic Icon";
				icon_type.label = "Icon Name";
				desc.label = "All lowercase with dashes between words, e.g list-add";
			} else {
				title = "New App Icon";
				icon_type.label = "App Name";
				// Hmm
				desc.label = "The reverse domain notation name, e.g. org.inkscape.Inkscape";
			}

			icon_title.notify["text"].connect(() => {
				accept_button.sensitive = icon_title.text.length > 0;
				if (mode == SYMBOLIC && !("-" in icon_title.text)) {
					icon_title.secondary_icon_name = "dialog-warning-symbolic";
					icon_title.secondary_icon_tooltip_text = "Expecting at least one '-'";
				} else if (mode == COLOUR && !("." in icon_title.text)) {
					icon_title.secondary_icon_name = "dialog-warning-symbolic";
					icon_title.secondary_icon_tooltip_text = "Expecting at least one '.'";
				} else {
					icon_title.secondary_icon_name = null;
					icon_title.secondary_icon_tooltip_text = null;
				}
			});
		}

		public Wizard (Window parent, Mode mode) {
			Object(transient_for: parent, mode: mode);
		}

		public override void response (int response) {
			if (response == ResponseType.ACCEPT) {
				new_icon.begin();
			} else {
				destroy();
			}
		}

		private async void launch (File file) {
			open(file);
			try {
				yield AppInfo.launch_default_for_uri_async(file.get_uri(), null);
			} catch (Error e) {
				critical("Failed to open %s: %s", file.get_basename(), e.message);
			}
		}

		private async void new_icon () requires (mode == COLOUR || mode == SYMBOLIC) {
			spin.visible = spin.active = true;

			if (mode == COLOUR) {
				yield launch(yield colour());
			} else {
				yield launch(yield symbolic());
			}

			Idle.add(() => {
				destroy();
				return Source.REMOVE;
			});
		}

		private async File colour () requires (mode == COLOUR) {
			var dest = File.new_for_path(Path.build_filename(location.file.get_path(), icon_title.text + ".svg"));
			var from = File.new_for_uri("resource:///org/gnome/IconPreview/templates/colour.svg");
			try {
				var p = dest.get_parent();
				if (!p.query_exists()) {
					p.make_directory_with_parents();
				}
				yield from.copy_async (dest, NONE);
				message("Copied %s -> %s", from.get_uri(), dest.get_uri());
			} catch (Error e) {
				critical ("Error: %s", e.message);
			}
			return dest;
		}

		private async File symbolic () requires (mode == SYMBOLIC) {
			var dest = File.new_for_path(Path.build_filename(location.file.get_path(), icon_title.text + "-symbolic.svg"));
			var from = File.new_for_uri("resource:///org/gnome/IconPreview/templates/symbolic.svg");
			try {
				var p = dest.get_parent();
				if (!p.query_exists()) {
					p.make_directory_with_parents();
				}
				yield from.copy_async (dest, NONE);
				message("Copied %s -> %s", from.get_uri(), dest.get_uri());
			} catch (Error e) {
				critical ("Error: %s", e.message);
			}
			return dest;
		}
	}
}

