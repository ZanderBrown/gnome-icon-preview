using Gtk;

namespace IconPreview {
	public class Symbolic : Box, Previewer {
		static string[] symbolics;

		private InfoBar info_bar = new InfoBar();
		private SymbolicPane light = new SymbolicPane();
		private SymbolicPane dark = new SymbolicPane();

		private Exporter exporter;

		private File _icon;
		public File previewing {
			get {
				return _icon;
			}
			set {
				_icon = value;
				exporter.update_regolar(null);
				exporter.update_nightly(null);
				exporter.update_symbolic(_icon);
				light.icon = dark.icon = new FileIcon(_icon);
				this.info_bar.visible = !_icon.get_basename().down().has_suffix("-symbolic.svg");
			}
		}

		public Symbolic(Exporter e) {
			Object(orientation: Orientation.VERTICAL);
			exporter = e;
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

			var rename_button = new Button.with_label(_("Rename"));
			rename_button.clicked.connect(() => {
				try {
					var icon_uri = this._icon.get_uri();
					var new_fileuri = icon_uri.substring(0, icon_uri.length - 4) + "-symbolic.svg";
					var destination = File.new_for_uri(new_fileuri);
					this._icon.move(destination, NONE);
					previewing = destination;
				} catch (Error e) {
					critical("Unable to rename the symbolic icon: %s", e.message);
				}
			});
			rename_button.show();
			info_bar.add_action_widget(rename_button, 0);

			var content_area = info_bar.get_content_area();
			var warning_label = new Label(_("The icon is not recoloring because the file name needs to end in “-symbolic”"));
			warning_label.halign = START;
			warning_label.set_valign(CENTER);
			warning_label.show();

			var warning_img = new Image.from_icon_name("dialog-warning-symbolic", LARGE_TOOLBAR);
			warning_img.show();
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
