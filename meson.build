project('gnome-icon-preview', 'vala', 'c', version: '0.1.0')


datadir = join_paths([ get_option('prefix'), get_option('datadir') ])
bindir = join_paths([ get_option('prefix'), 'bin' ])
vapidir = join_paths(meson.current_source_dir(), 'vapi')

add_project_arguments(['--vapidir', vapidir], language: 'vala')

conf = configuration_data()
conf.set_quoted('G_LOG_DOMAIN', 'org.gnome.IconPreview')
conf.set_quoted('PACKAGE_VERSION', meson.project_version())
conf.set('BINDIR', bindir)

conf.set_quoted('COMMIT_ID', 'unknown')
git_command = find_program('git', required: false)
if git_command.found()
  git_run = run_command('git', ['-C', meson.current_source_dir(), 'describe'])
  if git_run.returncode() == 0
    conf.set_quoted('COMMIT_ID', git_run.stdout().strip())
  else
    message('Failed to fetch commit')
  endif
else
  message('Git is unavailable')
endif

subdir('data')
subdir('src')

