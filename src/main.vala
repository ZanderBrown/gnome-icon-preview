using Gtk;

namespace IconPreview {
	public interface Previewer : Object {
		public abstract Icon previewing {get;set;}
		public abstract void shuffle();
		public abstract void export();
	}

	public class Application : Gtk.Application {
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
			set_accels_for_action ("win.open", { "<primary>O" });
			set_accels_for_action ("win.recents", { "<primary><shift>O" });
			set_accels_for_action ("win.refresh", { "<primary>R" });
			set_accels_for_action ("win.shuffle", { "<primary>S" });
			set_accels_for_action ("win.menu", { "F10" });
			set_accels_for_action ("win.quit", { "<primary>Q" });
		}
	}

	public int main (string[] args) {
		Environment.set_application_name("Icon Preview");
		Gtk.Window.set_default_icon_name("org.gnome.IconPreview");
		var app = new Application();
		return app.run(args);
	}
}
