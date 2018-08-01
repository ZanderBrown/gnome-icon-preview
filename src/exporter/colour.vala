using Gtk;

namespace IconPreview {
	[GtkTemplate (ui = "/org/gnome/IconPreview/exporter/colour.ui")]
	public class ColourExporter : Box, Exporter {
		[GtkChild]
		Image regular_image;

		[GtkChild]
		Label regular_label;

		[GtkChild]
		Image nightly_image;

		[GtkChild]
		Label nightly_label;

		public File file { get; set; }

		construct {
			notify["file"].connect(() => {
				regular_image.gicon = nightly_image.gicon = new FileIcon(file);
			});
		}

		[GtkCallback]
		private void regular () {
			close();
		}

		[GtkCallback]
		private void nightly () {
			close();
		}
	}
}
