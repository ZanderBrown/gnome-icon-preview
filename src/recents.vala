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

	[GtkTemplate (ui = "/org/gnome/IconPreview/recentrow.ui")]
	class RecentRow : ListBoxRow {
		[GtkChild]
		Label label;

		private Recent _recent;
		public Recent recent {
			construct {
				label.label = value.name;
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

	[GtkTemplate (ui = "/org/gnome/IconPreview/recents.ui")]
	public class Recents : Popover {
		[GtkChild]
		ListBox recent;

		RecentManager manager = RecentManager.get_default();
		GLib.ListStore model = new GLib.ListStore(typeof(Recent));

		construct {
			recent.bind_model(model, info => new RecentRow(info as Recent));
			manager.changed.connect(populate_model);
			populate_model();
		}

		public void open_file (File file) {
			manager.add_item(file.get_uri());
		}

		private void populate_model () {
			model.remove_all();
			foreach (var recent in manager.get_items()) {
				model.append(new Recent(recent));
			}
		}
	}
}

