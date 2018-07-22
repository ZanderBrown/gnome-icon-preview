using Gtk;

namespace IconPreview {
	public class Application : Gtk.Application {
		construct {
			application_id = "org.gnome.IconPreview";
			flags = HANDLES_OPEN;
		}

		public override void activate () {
			new Window2(this).present();
		}

		public override void open (File[] files, string hint) {
			foreach (var file in files) {
				var win = new Window2(this) {
					file = file
				};
				win.present();
			}
		}

		public override void startup () {
			base.startup();
			set_accels_for_action ("win.menu", { "F10" });
		}
	}

	public int main (string[] args) {
		Environment.set_application_name("Icon Preview");
		Gtk.Window.set_default_icon_name("org.gnome.IconPreview");
		var app = new Application();
		return app.run(args);
	}
}
