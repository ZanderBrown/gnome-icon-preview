desktop_file = i18n.merge_file(
  input: 'org.gnome.IconPreview.desktop.in',
  output: 'org.gnome.IconPreview.desktop',
  type: 'desktop',
  po_dir: '../po',
  install: true,
  install_dir: join_paths(get_option('datadir'), 'applications'),
)

palette_desktop_file = i18n.merge_file(
  input: 'org.gnome.IconPreview.Palette.desktop.in',
  output: 'org.gnome.IconPreview.Palette.desktop',
  type: 'desktop',
  po_dir: '../po',
  install: true,
  install_dir: join_paths(get_option('datadir'), 'applications'),
)

desktop_utils = find_program('desktop-file-validate', required: false)
if desktop_utils.found()
  test('Validate desktop file', desktop_utils,
    args: [desktop_file, palette_desktop_file]
  )
endif

appdata_file = i18n.merge_file(
  input: 'org.gnome.IconPreview.appdata.xml.in',
  output: 'org.gnome.IconPreview.appdata.xml',
  po_dir: '../po',
  install: true,
  install_dir: join_paths(get_option('datadir'), 'metainfo'),
)

appstream_util = find_program('appstream-util', required: false)
if appstream_util.found()
  validate_args = ['validate', appdata_file, '--nonet']
  test('Validate appstream file', appstream_util, args: validate_args)
endif

configure_file(input: 'org.gnome.IconPreview.service.in',
               output: 'org.gnome.IconPreview.service',
               configuration: conf,
               install_dir: join_paths([datadir,'dbus-1/services']))

configure_file(input: 'org.gnome.IconPreview.Palette.service.in',
               output: 'org.gnome.IconPreview.Palette.service',
               configuration: conf,
               install_dir: join_paths([datadir,'dbus-1/services']))

install_data('org.gnome.IconPreview.svg',
             install_dir: join_paths([datadir,'icons/hicolor/scalable/apps']))

install_data('org.gnome.IconPreview-symbolic.svg',
             install_dir: join_paths([datadir,'icons/hicolor/symbolic/apps']))

install_data('org.gnome.IconPreview.Palette.svg',
             install_dir: join_paths([datadir,'icons/hicolor/scalable/apps']))

install_data('org.gnome.IconPreview.Palette-symbolic.svg',
             install_dir: join_paths([datadir,'icons/hicolor/symbolic/apps']))

