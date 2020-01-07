project('app-icon-preview', 'vala', 'c', version: '1.0.1')

i18n = import('i18n')

datadir = join_paths(get_option('prefix'), get_option('datadir'))
bindir = join_paths(get_option('prefix'), get_option('bindir'))
localedir = join_paths(get_option('prefix'), get_option('localedir'))

application_id = 'org.gnome.design.AppIconPreview'
if get_option('profile') == 'development'
  profile = 'Devel'
  application_id += '.Devel'
else
  profile = ''
endif

conf = configuration_data()
conf.set_quoted('G_LOG_DOMAIN', application_id)
conf.set_quoted('PACKAGE_VERSION', meson.project_version())
conf.set_quoted('GETTEXT_PACKAGE', meson.project_name())
conf.set_quoted('LOCALE_DIR', localedir)
conf.set_quoted('APP_ID', application_id)
conf.set_quoted('PROFILE', profile)

add_global_arguments('-DGETTEXT_PACKAGE="@0@"'.format (meson.project_name()), language: 'c')
add_global_arguments('-DG_LOG_DOMAIN="@0@"'.format (application_id), language: 'c')
add_global_arguments('-DHANDY_USE_UNSTABLE_API', language: 'c')

subdir('data')
subdir('po')
subdir('src')

