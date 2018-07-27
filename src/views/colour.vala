using Gtk;

namespace IconPreview {
	[GtkTemplate (ui = "/org/gnome/IconPreview/colour.ui")]
	public class Colour : Box, Previewer {
		private File _icon;
		public File previewing {
			get {
				return _icon;
			}
			set {
				_icon = value;
			}
		}

		construct {

		}

		public void shuffle () {

		}

		public void export () {

		}
	}
}
