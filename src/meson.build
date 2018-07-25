deps = [
	dependency('gtk+-3.0')
]

config = configure_file(input: 'config.vala.in', output: 'config.vala', configuration: conf)

sources = [
	config,
	'pane.vala',
	'recents.vala',
	'symbolic.vala',
	'window.vala',
	'main.vala'
]

gnome = import('gnome')

sources += gnome.compile_resources('resources',
  'resource.gresource.xml'
)

executable('gnome-icon-preview', sources, dependencies: deps, install: true)
