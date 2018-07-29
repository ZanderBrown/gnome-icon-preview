using Gtk;

namespace IconPreview {
	public class Colour : Box, Previewer {
		const string RES_PATH = "/org/gnome/IconPreview/icons/";
		static List<string> colours;

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

		class construct {
			set_css_name("colour-view");
		}

		static construct {
			try {
				var icons = resources_enumerate_children(RES_PATH, NONE);
				foreach (var icon in icons) {
					colours.append(icon);
				}
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

			shuffle();
		}

		public void shuffle () {
			var samples_names = new string[5];
			var samples = new Icon[5];

			var i = 0;
			while (i < 5) {
				var pos = Random.int_range(0, (int32) colours.length());
				var proposed = colours.nth_data(pos);
				if (proposed in samples_names) {
					continue;
				}
				samples_names[i] = proposed;
				i++;
			}

			for (var j = 0; j < 5; j++) {
				samples[j] = new FileIcon(File.new_for_uri("resource:/" + RES_PATH + samples_names[j]));
			}

			light.load_samples(samples);
			dark.load_samples(samples);
		}

		public void export () {
			message("TODO");
		}
	}
}
