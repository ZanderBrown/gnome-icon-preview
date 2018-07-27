using Gtk;

namespace IconPreview {
	public class Symbolic : Box, Previewer {
		private Pane light = new Pane();
		private Pane dark = new Pane();

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
			light.shuffle();
			dark.shuffle();
		}

		public void export () {
			message("TODO");
		}
	}
}
