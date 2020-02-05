desktop_conf = configuration_data()
desktop_conf.set('icon', application_id)
desktop_file = i18n.merge_file(
  input: configure_file(
    input: 'org.gnome.design.AppIconPreview.desktop.in.in',
    output: '@BASENAME@',
    configuration: desktop_conf
  ),
  output: '@0@.desktop'.format(application_id),
  type: 'desktop',
  po_dir: '../po',
  install: true,
  install_dir: join_paths(get_option('datadir'), 'applications'),
)

desktop_utils = find_program('desktop-file-validate', required: false)
if desktop_utils.found()
  test('Validate desktop file', desktop_utils,
    args: [desktop_file]
  )
endif

appdata_conf = configuration_data()
appdata_conf.set('app-id', application_id)
appdata_file = i18n.merge_file(
  input: configure_file(
    input: 'org.gnome.design.AppIconPreview.metainfo.xml.in.in',
    output: '@BASENAME@',
    configuration: appdata_conf
  ),
  output: '@0@.metainfo.xml'.format(application_id),
  po_dir: '../po',
  install: true,
  install_dir: join_paths(get_option('datadir'), 'metainfo'),
)

appstream_util = find_program('appstream-util', required: false)
if appstream_util.found()
  validate_args = ['validate', appdata_file, '--nonet']
  test('Validate appstream file', appstream_util, args: validate_args)
endif

service_conf = configuration_data()
service_conf.set('app-id', application_id)
service_conf.set('bindir', bindir)
configure_file(input: 'org.gnome.design.AppIconPreview.service.in',
               output: '@0@.service'.format(application_id),
               configuration: service_conf,
               install_dir: join_paths([datadir,'dbus-1/services']))

install_data('@0@.svg'.format(application_id),
             install_dir: join_paths([datadir,'icons/hicolor/scalable/apps']))

install_data('org.gnome.design.AppIconPreview-symbolic.svg',
             rename: '@0@-symbolic.svg'.format(application_id),
             install_dir: join_paths([datadir,'icons/hicolor/symbolic/apps']))

