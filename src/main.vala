using Gtk;

namespace IconPreview {
	public enum Mode {
		INITIAL,
		SYMBOLIC,
		COLOUR
	}

	// Adapted from one of the gtk demos
	public void apply_css(Widget widget, CssProvider provider) {
		var context = widget.get_style_context();
		StyleProvider existing = widget.get_data("it-style-provider");
		if (existing != null) {
			context.remove_provider(existing);
		}
		context.add_provider(provider, uint.MAX - 10);
		widget.set_data("it-style-provider", provider);
		if (widget is Container) {
			(widget as Container).forall(child => apply_css(child, provider));
		}
	}

	public interface Previewer : Widget {
		public abstract File previewing { get; set; }
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

	public class Application : Dazzle.Application {
		const GLib.ActionEntry[] entries = {
			{ "new-window", new_window },
			{ "quit",  quit  }
		};

		construct {
			// Bind the actions
			add_action_entries(entries, this);

			application_id = APP_ID;
			flags = HANDLES_OPEN | HANDLES_COMMAND_LINE;

			add_main_option ("palette", 'p', IN_MAIN, NONE, _("no longer supported"), null);
		}

		// Open a new window (app.new-window)
		private void new_window () {
			new Window(this).show();
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
			set_resource_base_path("/org/gnome/design/IconPreview");
			base.startup();

			var styles = new CssProvider();
			styles.load_from_resource("/org/gnome/design/IconPreview/gtk/style.css");
			// Use of uint.MAX isn't ideal but we are effectively in an arms race
			StyleContext.add_provider_for_screen(Gdk.Screen.get_default(), styles, uint.MAX);

			set_accels_for_action ("win.open", { "<primary>O" });
			set_accels_for_action ("app.new-window", { "<primary>N" });
			set_accels_for_action ("win.recents", { "<primary><shift>O" });
			set_accels_for_action ("win.refresh", { "<primary>R" });
			set_accels_for_action ("win.export", { "<primary>E" });
			set_accels_for_action ("win.shuffle", { "<primary>S" });
			set_accels_for_action ("win.screenshot", { "<primary><alt>s" });
			set_accels_for_action ("win.copy-screenshot", { "<primary><alt>c" });
			set_accels_for_action ("win.menu", { "F10" });
			set_accels_for_action ("win.fullscreen", { "F11" });
			set_accels_for_action ("app.quit", { "<primary>Q" });
		}

		public override int command_line (ApplicationCommandLine cli) {
			var options = cli.get_options_dict();

			// If opening the palette directly
			if (options.contains("palette")) {
				cli.printerr(_("Palette is all grown up!\n" +
				               "It’s now available separately as org.gnome.zbrown.Palette"));

				// Don't activate normally
				return 0;
			}

			activate();

			return base.command_line(cli);
		}
	}

	public int main (string[] args) {
		Gtk.Window.set_default_icon_name(APP_ID);

		Intl.setlocale (LocaleCategory.ALL, "");
		Intl.bindtextdomain (GETTEXT_PACKAGE, LOCALE_DIR);
		Intl.bind_textdomain_codeset (GETTEXT_PACKAGE, "UTF-8");
		Intl.textdomain (GETTEXT_PACKAGE);

		Environment.set_application_name(_("Icon Preview"));

		var app = new Application();
		return app.run(args);
	}
}
