// (C) 2018 Zander Brown

using Gtk;

class IconPlayWindow : Window {
	private Image standard = new Image.from_icon_name("start-here", DIALOG);
	private Image symbolic = new Image.from_icon_name("start-here-symbolic", DIALOG);

	private Entry icon_pick = new Entry();
	private SpinButton size = new SpinButton.with_range(16, 2048, 1);
	private ColorButton colour = new ColorButton();
	
	construct {
		height_request = 300;
		width_request = 400;
		var wrap = new Box(VERTICAL, 0);
		wrap.expand = true;
		
		var controls = new Box(HORIZONTAL, 15);
		controls.border_width = 5;
		controls.valign = START;
		controls.halign = CENTER;
		controls.add(new Label("Icon"));
		controls.add(icon_pick);
		icon_pick.text = standard.icon_name;
		icon_pick.changed.connect(() => {
			standard.icon_name = icon_pick.text;
			symbolic.icon_name = icon_pick.text + "-symbolic";
		});
		icon_pick.completion = new EntryCompletion ();

		controls.add(new Label("Size"));
		controls.add(size);
		size.value = (double) (standard.pixel_size = symbolic.pixel_size = 128);
		size.notify["value"].connect(() => {
			standard.pixel_size = symbolic.pixel_size = (int) size.value;
			standard.icon_name = icon_pick.text;
			symbolic.icon_name = icon_pick.text + "-symbolic";
		});
		controls.add(new Label("Colour"));
		controls.add(colour);
		var provider = new CssProvider();
		var tmpl = "image { color:	%s; }";
		colour.color_set.connect(() => {
			try {
				provider.load_from_data(tmpl.printf(colour.rgba.to_string()));
			} catch (Error e) {
				message("Couldn't set colour: %s", e.message);
			}
		});
		colour.rgba = get_style_context().get_color(NORMAL);
		StyleContext.add_provider_for_screen (Gdk.Screen.get_default (), provider, STYLE_PROVIDER_PRIORITY_USER);
		wrap.pack_start(controls, false, false);

		var icons = new Box(HORIZONTAL, 0);
		icons.expand = true;
		icons.valign = CENTER;
		icons.homogeneous = true;
		icons.add(standard);
		icons.add(symbolic);
		var scroll = new ScrolledWindow(null, null);
		scroll.add(icons);
		wrap.pack_end(scroll, true, true);

		add(wrap);

		var list_store = new Gtk.ListStore (1, typeof (string));
		var iter = TreeIter();
		foreach (var icon in IconTheme.get_default().list_icons(null)) {
			if (!icon.has_suffix("symbolic")) {
				list_store.append (out iter);
				list_store.set (iter, 0, icon);
			}
		}
		icon_pick.completion.model = list_store;
		icon_pick.completion.text_column = 0;
	}
}

