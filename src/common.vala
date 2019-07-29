using Gtk;

namespace IconPreview {
	public void about_app (Gtk.Window parent) {
		var authors = new string[] {"Zander Brown", "Bilal Elmoussaoui"};
		var artists = new string[] {"Tobias Bernard"};
		var helpers = new string[] {"Jordan Petridis"};
		var dlg = new AboutDialog () {
			transient_for = parent,
			modal = true,
			program_name = Environment.get_application_name(),
			logo_icon_name = Gtk.Window.get_default_icon_name(),
			version = PACKAGE_VERSION,
			copyright = _("Copyright © 2018-19 Zander Brown"),
			license_type = GPL_3_0,
			authors = authors,
			artists = artists,
			website = "https://gitlab.gnome.org/World/design/icon-tool/",
			website_label = _("Repository")
		};
		dlg.add_credit_section(_("Kept sane by"), helpers);
		dlg.show();
	}
}
