using Gtk;

namespace IconPreview {
	class Recent : Object {
		public string name;
		public string uri;

		public Recent (RecentInfo info) {
			name = info.get_display_name();
			uri = info.get_uri();
		}
	}

	[GtkTemplate (ui = "/org/gnome/design/AppIconPreview/recentrow.ui")]
	class RecentRow : ListBoxRow {
		[GtkChild]
		Label label;
		[GtkChild]
		Image image;

		private Recent _recent;
		public Recent recent {
			construct {
				label.label = value.name;
				this.tooltip_text = value.name;

				var svg = new Rsvg.Handle.from_gfile_sync(File.new_for_uri(value.uri), FLAGS_NONE);
				var hicolor = create_tmp_file ("#hicolor");
				render_by_id(svg, "#hicolor", hicolor, 32);

				var gicon = new FileIcon(hicolor);
				image.set_from_gicon(gicon, DND);

				_recent = value;
			}

			get {
				return _recent;
			}
		}

		public RecentRow (Recent info) {
			Object(recent: info);
		}
	}

	[GtkTemplate (ui = "/org/gnome/design/AppIconPreview/recents.ui")]
	public class Recents : Popover {
		[GtkChild]
		ListBox recent;

		RecentManager manager = RecentManager.get_default();
		GLib.ListStore model = new GLib.ListStore(typeof(Recent));

		public signal void open (File file);

		public virtual signal void opened (File file) {
			var uri = file.get_uri();
			if (!manager.has_item(uri)) {
				manager.add_item(uri);
			}
		}

		construct {
			recent.set_header_func((before, after) => {
			  	if (after != null) {
			        var separator = new Gtk.Separator (Gtk.Orientation.HORIZONTAL);
			        separator.show();
			        before.set_header(separator);
			    }
			});

			recent.bind_model(model, info => new RecentRow(info as Recent));
			manager.changed.connect(populate_model);
			populate_model();
		}

		private void populate_model () {
			model.remove_all();
			foreach (var recent in manager.get_items()) {
				var file = File.new_for_uri(recent.get_uri());
				if (file.query_exists()) {
					var svg = new Rsvg.Handle.from_gfile_sync(file, FLAGS_NONE);
					if (svg.has_sub("#hicolor")) {
						model.append(new Recent(recent));
					}
				}
			}
		}

		[GtkCallback]
		private void activated (ListBox _box, ListBoxRow row) {
			popdown();
			Idle.add(() => {
				open(File.new_for_uri((row as RecentRow).recent.uri));
				return Source.REMOVE;
			});
		}
	}
}

