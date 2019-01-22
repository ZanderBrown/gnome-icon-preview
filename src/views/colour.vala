using Gtk;

namespace IconPreview {
	public class Colour : ScrolledWindow, Previewer {
		const string RES_PATH = "/org/gnome/IconPreview/icons/";
		static string[] colours;

		private ColourPane light = new ColourPane();
		private ColourPane dark = new ColourPane();

		private File _icon;
		public File previewing {
			get {
				return _icon;
			}
			set {
				_icon = value;
				light.icon = dark.icon = new FileIcon(_icon);
			}
		}

		private ColourExporter _export = new ColourExporter();
		public Exporter exporter {
			owned get {
				return _export;
			}
		}

		class construct {
			set_css_name("colour-view");
		}

		static construct {
			try {
				colours = resources_enumerate_children(RES_PATH, NONE);
			} catch (Error e) {
				critical("Failed to load sample icons: %s", e.message);
			}
		}

		construct {
			hscrollbar_policy = NEVER;
			min_content_height = 520;

			light.theme = "Adwaita";
			dark.theme = "Adwaita-dark";

			var view = new Viewport(null, null);
			view.shadow_type = NONE;
			view.show();

			var box = new Columns ();
			box.add(light);
			box.add(dark);
			box.show();
			view.add(box);

			add(view);

			bind_property("previewing", _export, "file");

			shuffle();
		}

		public void shuffle () {
			var samples_names = random_selection(colours, 9);
			var samples = new Icon[9];

			for (var j = 0; j < 9; j++) {
				samples[j] = new FileIcon(File.new_for_uri("resource:/" + RES_PATH + samples_names[j]));
			}

			light.load_samples(samples);
			dark.load_samples(samples);
		}
	}
}
