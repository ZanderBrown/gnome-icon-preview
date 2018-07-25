using Gtk;

namespace IconPreview {
	[GtkTemplate (ui = "/org/gnome/IconPreview/inital.ui")]
	class InitalState : Grid {
		[GtkChild]
		Box drop;
	}
}

