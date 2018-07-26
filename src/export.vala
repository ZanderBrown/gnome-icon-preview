using Gtk;

namespace IconPreview {
	[GtkTemplate (ui = "/org/gnome/IconPreview/export.ui")]
	public class Export : Popover {
		public Mode mode { get; set; default = INITIAL; }
		// TODO
	}
}
