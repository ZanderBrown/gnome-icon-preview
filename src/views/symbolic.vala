using Gtk;

namespace IconPreview {
	public class Symbolic : Box, Previewer {
		static string[] symbolics;

		private InfoBar info_bar = new InfoBar();
		private SymbolicPane light = new SymbolicPane();
		private SymbolicPane dark = new SymbolicPane();

		private File _icon;
		public File previewing {
			get {
				return _icon;
			}
			set {
				_icon = value;
				light.icon = dark.icon = new FileIcon(_icon);
				if (_icon.get_basename().contains("-symbolic")) {
					this.info_bar.hide();
				} else {
					this.info_bar.show_all();
				}
			}
		}
		public Symbolic () {
			Object(orientation: Orientation.VERTICAL);
		}

		public Exporter exporter {
			owned get {
				return new SymbolicExporter();
			}
		}

		class construct {
			set_css_name("symbolic-view");
		}

		static construct {
			var tmp = new List<string>();
			foreach (var icon in IconTheme.get_default().list_icons(null)) {
				if (icon.has_suffix("symbolic")) {
					tmp.append(icon);
				}
			}
			var len = tmp.length();
			symbolics = new string[len];
			for (var i = 0; i < len; i++) {
				symbolics[i] = tmp.nth_data(i);
			}
		}

		construct {
			var panes_container = new Box(HORIZONTAL, 0);
			light.theme = "Adwaita";
			dark.theme = "Adwaita-dark";
			panes_container.pack_start(light);
			panes_container.pack_end(dark);

			var warning_label = new Label(_("The icon is not recoloring because the file name needs to end in \"-symbolic\""));
			warning_label.set_halign(START);
			warning_label.set_valign(CENTER);

			var content_area = info_bar.get_content_area();
			var warning_img = new Image.from_icon_name("dialog-warning-symbolic", LARGE_TOOLBAR);
			content_area.add(warning_img);
			content_area.add(warning_label);
			content_area.margin = 6;

			info_bar.set_message_type(WARNING);

			pack_start(info_bar);
			pack_start(panes_container);
			panes_container.show();

			shuffle();
		}

		public void shuffle () {
			var samples_names = random_selection(symbolics, 20);
			var samples = new Icon[20];

			for (var j = 0; j < 20; j++) {
				samples[j] = new ThemedIcon(samples_names[j]);
			}

			light.load_samples(samples);
			dark.load_samples(samples);
		}
	}
}
