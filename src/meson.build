deps = [
	dependency('gtk+-3.0')
]

config = configure_file(input: 'config.vala.in', output: 'config.vala', configuration: conf)

sources = [
	config,
	'icons.vala',
	'main.vala'
]

executable('gnome-icon-preview', sources, dependencies: deps, install: true)
