using Gtk;

namespace IconPreview {

	public Cairo.Surface? render_by_id (Rsvg.Handle svg, string id, File file, int output_size) {
		if (svg.has_sub(id)) {
			Rsvg.Rectangle size;
			Rsvg.Rectangle viewport = { 0.0, 0.0, svg.width, svg.height };
			svg.get_geometry_for_layer (id, viewport, null, out size);
			var surface = new Cairo.SvgSurface(file.get_path(), output_size, output_size);
			surface.set_document_unit (Cairo.SvgUnit.PX);
			var cr = new Cairo.Context(surface);
			cr.scale(output_size/size.width, output_size/size.height);
			cr.translate(-size.x, -size.y);
			svg.render_cairo(cr);
			return surface;
		}
		return null;
	}

	public File create_tmp_file (string id) {
		FileIOStream stream;
		return File.new_tmp("XXXXXX-" + id.substring(1, -1) +".svg", out stream);
	}

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
			translator_credits = _("translator-credits"),
			website = "https://gitlab.gnome.org/World/design/app-icon-preview/",
			website_label = _("Repository")
		};
		dlg.add_credit_section(_("Kept sane by"), helpers);
		dlg.show();
	}
}
