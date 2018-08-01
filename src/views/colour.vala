using Gtk;

namespace IconPreview {
	public class Colour : Box, Previewer {
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
			homogeneous = true;

			light.theme = "Adwaita";
			dark.theme = "Adwaita-dark";
			pack_start(light);
			pack_end(dark);

			bind_property("previewing", _export, "file");

			shuffle();
		}

		public void shuffle () {
			var samples_names = random_selection(colours, 5);
			var samples = new Icon[5];

			for (var j = 0; j < 5; j++) {
				samples[j] = new FileIcon(File.new_for_uri("resource:/" + RES_PATH + samples_names[j]));
			}

			light.load_samples(samples);
			dark.load_samples(samples);
		}
	}
}
