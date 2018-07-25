using Gtk;

namespace IconPreview {
	public class Symbolic : Box, Previewer {
		private Pane light = new Pane();
		private Pane dark = new Pane();

		public Icon previewing {
			get {
				return light.icon;
			}
			set {
				light.icon = dark.icon = value;
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
