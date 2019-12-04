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

	[GtkTemplate (ui = "/org/gnome/design/AppIconPreview/recents.ui")]
	public class Recents : Popover {
		[GtkChild]
		ListBox recent;

		RecentManager manager = RecentManager.get_default();
		GLib.ListStore model = new GLib.ListStore(typeof(Recent));

		public signal void open (File file);

		public virtual signal void opened (File file) {
			manager.add_item(file.get_uri());
		}

		construct {
			recent.bind_model(model, info => new RecentRow(info as Recent));
			manager.changed.connect(populate_model);
			populate_model();
		}

		private void populate_model () {
			model.remove_all();
			foreach (var recent in manager.get_items()) {
				model.append(new Recent(recent));
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

