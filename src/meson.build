deps = [
	dependency('gtk+-3.0'),
	dependency('librsvg-2.0'),
	dependency('libdazzle-1.0'),
	dependency('libhandy-0.0')
]

config = configure_file(input: 'config.vala.in', output: 'config.vala', configuration: conf)

sources = [
	config,
	'recents.vala',
	'views/utils.vala',
	'views/colourpane.vala',
	'views/colour.vala',
	'views/symbolicpane.vala',
	'views/symbolic.vala',
	'exporter/colour.vala',
	'exporter/symbolic.vala',
	'wizard.vala',
	'window.vala',
	'main.vala'
]

gnome = import('gnome')

sources += gnome.compile_resources('resources',
  'resource.gresource.xml'
)

sources += gnome.compile_resources('icons',
  'icons/icons.gresource.xml',
  source_dir: 'icons/'
)

sources += gnome.compile_resources('wallpapers',
  'wallpapers/wallpapers.gresource.xml',
  source_dir: 'wallpapers/'
)

executable('gnome-icon-preview', sources, dependencies: deps, install: true)

