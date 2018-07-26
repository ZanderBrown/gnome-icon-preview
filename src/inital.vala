using Gtk;

namespace IconPreview {
	enum Target {
		INT32,
		STRING,
		ROOTWIN
	}

	[GtkTemplate (ui = "/org/gnome/IconPreview/inital.ui")]
	class InitalState : Grid {
		/*
		[GtkChild]
		Box zone;

		private const TargetEntry[] target_list = {
			{ "INTEGER",    0, Target.INT32 },
			{ "STRING",     0, Target.STRING },
			{ "text/plain", 0, Target.STRING },
			{ "application/x-rootwindow-drop", 0, Target.ROOTWIN }
		};

		construct {
			drag_dest_set (zone, DROP, target_list, COPY);
			zone.drag_motion.connect(motion);
			zone.drag_leave.connect(leave);
			zone.drag_drop.connect(drop);
			zone.drag_data_received.connect(received);
		}

		private bool motion (Widget widget, Gdk.DragContext context, int x, int y, uint time) {
			return false;
		}

		private void leave (Widget widget, Gdk.DragContext context, uint time) {
			print ("%s: leave\n", widget.name);
		}

		private bool drop (Widget widget, Gdk.DragContext context, int x, int y, uint time) {
			print ("%s: drop\n", widget.name);

			bool is_valid_drop_site = true;

			if (context.list_targets() != null) {
				var target_type = (Gdk.Atom) context.list_targets().nth_data (Target.INT32);
				Gtk.drag_get_data (widget, context, target_type, time);
			} else {
				is_valid_drop_site = false;
			}

			return is_valid_drop_site;
		}

		private void received (Widget widget, Gdk.DragContext context, int x, int y, SelectionData selection_data, uint target_type, uint time) {
			bool dnd_success = false;
			bool delete_selection_data = false;

			print ("%s: received\n", widget.name);

			if ((selection_data != null) && (selection_data.get_length() >= 0)) {
				print (" Receiving ");
				switch (target_type) {
					case Target.INT32:
						long* data = (long*) selection_data.get_data();
						print ("integer: %ld", (*data));
						dnd_success = true;
						break;
					case Target.STRING:
						print ("string: %s", (string) selection_data.get_data());
						dnd_success = true;
						break;
					default:
						print ("nothing good");
						break;
				}
				print (".\n");
			}

			if (dnd_success == false) {
				print ("DnD data transfer failed!\n");
			}

			drag_finish (context, dnd_success, delete_selection_data, time);
		}
		*/
	}
}

