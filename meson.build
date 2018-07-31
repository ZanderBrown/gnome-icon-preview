project('gnome-icon-preview', 'vala', 'c', version: '0.1.0')


datadir = join_paths([ get_option('prefix'), get_option('datadir') ])
bindir = join_paths([ get_option('prefix'), 'bin' ])

conf = configuration_data()
conf.set_quoted('G_LOG_DOMAIN', 'org.gnome.IconPreview')
conf.set_quoted('PACKAGE_VERSION', meson.project_version())
conf.set('BINDIR', bindir)

subdir('data')
subdir('src')

