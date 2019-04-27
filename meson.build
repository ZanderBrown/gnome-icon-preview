project('gnome-icon-preview', 'vala', 'c', version: '0.9.4')

i18n = import('i18n')

datadir = join_paths([ get_option('prefix'), get_option('datadir') ])
bindir = join_paths([ get_option('prefix'), 'bin' ])
localedir = join_paths(get_option('prefix'), get_option('localedir'))

conf = configuration_data()
conf.set_quoted('G_LOG_DOMAIN', 'org.gnome.IconPreview')
conf.set_quoted('PACKAGE_VERSION', meson.project_version())
conf.set_quoted('GETTEXT_PACKAGE', meson.project_name())
conf.set_quoted('LOCALE_DIR', localedir)
conf.set('BINDIR', bindir)

add_global_arguments('-DGETTEXT_PACKAGE="@0@"'.format (meson.project_name()), language: 'c')
add_global_arguments('-DG_LOG_DOMAIN="@0@"'.format ('org.gnome.IconPreview'), language: 'c')
add_global_arguments('-DHANDY_USE_UNSTABLE_API', language: 'c')

subdir('data')
subdir('po')
subdir('src')

