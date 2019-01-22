using Gtk;

namespace IconPreview {
	public class Columns : Container {
		private List<Widget> children;

		public bool collapsed { public get; private set; }

		construct {
			set_has_window (false);
			set_redraw_on_allocate (false);
			set_can_focus (false);
		}

		static construct {
			set_accessible_role (PANEL);
			set_css_name ("columns");
		}

		public override void add (Widget child) {
			children.append (child);

			child.set_parent (this);
		}

		public override void remove (Widget child) {
			children.remove (child);

			child.set_parent (null);
		}

		public override void forall_internal (bool internals, Gtk.Callback cb) {
			foreach (var child in children) {
				cb (child);
			}
		}

		public override void size_allocate (Allocation alloc) {
			set_allocation (alloc);

			int min = 0;
			foreach (var child in children) {
				int c_min = 0;

				if (!child.visible) continue;

				child.get_preferred_width (out c_min, null);

				min += c_min;
			}

			if (min > alloc.width) {
				message ("Time to collapse");
				collapsed = true;
			} else {
				collapsed = false;
			}

			int offset = 0;
			int c_width = alloc.width / (int) children.length();
			foreach (var child in children) {
				Allocation c_alloc = alloc;

				if (!child.visible) continue;

				c_alloc.x += offset;
				c_alloc.width = c_width;

				child.size_allocate (c_alloc);

				offset += c_width;
			}
		}

		public override void get_preferred_width (out int min, out int nat) {
			int _nat = 0;
			int _min = 0;

			foreach (var child in children) {
				int c_nat, c_min;

				if (!child.visible) continue;

				child.get_preferred_width (out c_min, out c_nat);

				_nat += c_nat;

				if (c_min > _min)
					_min = c_min;

			}

			nat = _nat;
			/*min = _min;*/
			min = nat;
		}

		public override void get_preferred_height (out int min, out int nat) {
			int _nat = 0;
			int _min = 0;

			foreach (var child in children) {
				int c_nat, c_min;

				if (!child.visible) continue;

				child.get_preferred_height (out c_min, out c_nat);

				if (c_nat > _nat)
					_nat = c_nat;

				if (c_min > _min)
					_min = c_min;
			}

			nat = _nat;
			min = _min;
		}
	}
}
