configure_file(input: 'org.gnome.IconPreview.desktop.in',
               output: 'org.gnome.IconPreview.desktop',
               configuration: conf,
               install_dir: join_paths([datadir,'applications']))

configure_file(input: 'org.gnome.IconPreview.service.in',
               output: 'org.gnome.IconPreview.service',
               configuration: conf,
               install_dir: join_paths([datadir,'dbus-1/services']))

install_data('org.gnome.IconPreview.svg',
             install_dir: join_paths([datadir,'icons/hicolor/scalable/apps']))

install_data('org.gnome.IconPreview-symbolic.svg',
             install_dir: join_paths([datadir,'icons/hicolor/symbolic/apps']))
