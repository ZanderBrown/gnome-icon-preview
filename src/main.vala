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
		construct {
			application_id = "org.gnome.IconPreview";
			flags = HANDLES_OPEN;
		}

		public override void activate () {
			new Window(this).present();
		}

		public override void open (File[] files, string hint) {
			foreach (var file in files) {
				var win = new Window(this) {
					file = file
				};
				win.present();
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
			set_accels_for_action ("win.quit", { "<primary>Q" });
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
