desktop_file = i18n.merge_file(
  input: 'org.gnome.design.IconPreview.desktop.in',
  output: '@BASENAME@',
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

appdata_file = i18n.merge_file(
  input: 'org.gnome.design.IconPreview.appdata.xml.in',
  output: '@BASENAME@',
  po_dir: '../po',
  install: true,
  install_dir: join_paths(get_option('datadir'), 'metainfo'),
)

appstream_util = find_program('appstream-util', required: false)
if appstream_util.found()
  validate_args = ['validate', appdata_file, '--nonet']
  test('Validate appstream file', appstream_util, args: validate_args)
endif

configure_file(input: 'org.gnome.design.IconPreview.service.in',
               output: '@BASENAME@',
               configuration: conf,
               install_dir: join_paths([datadir,'dbus-1/services']))

install_data('org.gnome.design.IconPreview.svg',
             install_dir: join_paths([datadir,'icons/hicolor/scalable/apps']))

install_data('org.gnome.design.IconPreview-symbolic.svg',
             install_dir: join_paths([datadir,'icons/hicolor/symbolic/apps']))

