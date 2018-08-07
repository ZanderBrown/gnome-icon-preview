using Gtk;

namespace IconPreview {
	[GtkTemplate (ui = "/org/gnome/IconPreview/initial.ui")]
	public class InitialState : Grid {
		class construct {
			set_css_name("initial-state");
		}
	}
}

