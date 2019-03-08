using Gtk;

namespace IconPreview {
	public enum Mode {
		INITIAL,
		SYMBOLIC,
		COLOUR
	}

	public interface Previewer : Widget {
		public abstract File previewing { get; set; }
		public abstract Exporter exporter { owned get; }
		public abstract void shuffle();
		public virtual Gdk.Pixbuf screenshot () {
			var w = get_allocated_width();
			var h = get_allocated_height();
			var surface = new Cairo.ImageSurface (ARGB32, w, h);
			var context = new Cairo.Context (surface);
			draw(context);
			return Gdk.pixbuf_get_from_surface (surface, 0, 0, w, h);
		}
	}

	public interface Exporter : Widget {
		public signal void close();
	}

	public class Application : Dazzle.Application {
		const GLib.ActionEntry[] entries = {
			{ "palette", palette },
			{ "quit",  quit  }
		};

		construct {
			// Bind the actions
			add_action_entries(entries, this);

			application_id = "org.gnome.IconPreview";
			flags = HANDLES_OPEN | HANDLES_COMMAND_LINE;

			// Open the palette instead of a normal window
			add_main_option ("palette", 'p', IN_MAIN, NONE, _("Open colour palette"), null);
		}

		public void about (Gtk.Window parent) {
			var authors = new string[] {"Zander Brown", "Bilal Elmoussaoui"};
			var artists = new string[] {"Tobias Bernard"};
			show_about_dialog (parent,
				program_name: _("Icon Preview"),
				logo_icon_name: "org.gnome.IconPreview",
				version: PACKAGE_VERSION,
				copyright: _("Copyright © 2018 Zander Brown"),
				license_type: License.GPL_3_0,
				authors: authors,
				artists: artists,
				website: "https://gitlab.gnome.org/ZanderBrown/icon-tool/",
				website_label: _("Repository"));
		}

		// Handler for app.palette
		private void palette () {
			new Palette(this).show();
		}

		public override void activate () {
			new Window(this).show();
		}

		public override void open (File[] files, string hint) {
			foreach (var file in files) {
				var win = new Window(this) {
					file = file
				};
				win.show();
			}
		}

		public override void startup () {
			base.startup();

			var styles = new CssProvider();
			styles.load_from_resource("/org/gnome/IconPreview/gtk/style.css");
			// Use of uint.MAX isn't ideal but we are effectivly in an arms race
			StyleContext.add_provider_for_screen(Gdk.Screen.get_default(), styles, uint.MAX);

			set_accels_for_action ("win.open", { "<primary>O" });
			set_accels_for_action ("win.new-window", { "<primary>N" });
			set_accels_for_action ("win.recents", { "<primary><shift>O" });
			set_accels_for_action ("win.refresh", { "<primary>R" });
			set_accels_for_action ("win.export", { "<primary>E" });
			set_accels_for_action ("win.shuffle", { "<primary>S" });
			set_accels_for_action ("win.screenshot", { "<primary><alt>s" });
			set_accels_for_action ("win.menu", { "F10" });
			set_accels_for_action ("win.fullscreen", { "F11" });
			set_accels_for_action ("app.quit", { "<primary>Q" });
		}

		public override int command_line (ApplicationCommandLine cli) {
			var options = cli.get_options_dict();

			// If opening the palette directly
			if (options.contains("palette")) {
				palette();

				// Don't activate normally
				return 0;
			}

			// Handle files ext
			return base.command_line(cli);
		}
	}

	public int main (string[] args) {
		Gtk.Window.set_default_icon_name("org.gnome.IconPreview");

		Intl.setlocale (LocaleCategory.ALL, "");
		Intl.bindtextdomain (GETTEXT_PACKAGE, LOCALE_DIR);
		Intl.bind_textdomain_codeset (GETTEXT_PACKAGE, "UTF-8");
		Intl.textdomain (GETTEXT_PACKAGE);

		Environment.set_application_name(_("Icon Preview"));

		var app = new Application();
		return app.run(args);
	}
}
