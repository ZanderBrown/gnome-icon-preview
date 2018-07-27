using Gtk;

namespace IconPreview {
	public class Colour : Box, Previewer {
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

		construct {
			light.theme = "Adwaita";
			dark.theme = "Adwaita-dark";
			pack_start(light);
			pack_end(dark);
		}

		public void shuffle () {

		}

		public void export () {
			message("TODO");
		}
	}
}
