using Gtk;

namespace IconPreview {
	public class Symbolic : Box, Previewer {
		static string[] symbolics;

		private SymbolicPane light = new SymbolicPane();
		private SymbolicPane dark = new SymbolicPane();

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
			set_css_name("symbolic-view");
		}

		static construct {
			var tmp = new List<string>();
			foreach (var icon in IconTheme.get_default().list_icons(null)) {
				if (icon.has_suffix("symbolic")) {
					tmp.append(icon);
				}
			}
			var len = tmp.length();
			symbolics = new string[len];
			for (var i = 0; i < len; i++) {
				symbolics[i] = tmp.nth_data(i);
			}
		}

		construct {
			light.theme = "Adwaita";
			dark.theme = "Adwaita-dark";
			pack_start(light);
			pack_end(dark);

			shuffle();
		}

		public void shuffle () {
			var samples_names = random_selection(symbolics, 20);
			var samples = new Icon[20];

			for (var j = 0; j < 20; j++) {
				samples[j] = new ThemedIcon(samples_names[j]);
			}

			light.load_samples(samples);
			dark.load_samples(samples);
		}

		public void export () {
			message("TODO");
		}
	}
}
