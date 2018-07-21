using Gtk;

namespace IconPreview { 
	class Application : Gtk.Application {
		construct {
			application_id = "org.gnome.IconPreview";
			flags = FLAGS_NONE;
		}

		public override void activate () {
			var win = new IconPlayWindow() {
				application = this
			};
			win.destroy.connect(quit);
			win.show_all();
		}

		public override void startup () {
			base.startup();
		}
	}

	public int main (string[] args) {
		Environment.set_application_name("Icon Preview");
		var app = new Application();
		return app.run(args);
	}
}