project('icon-tool', 'vala', 'c')


datadir = join_paths([ get_option('prefix'), get_option('datadir') ])
bindir = join_paths([ get_option('prefix'), 'bin' ])

conf = configuration_data()
conf.set_quoted('G_LOG_DOMAIN', 'org.gnome.IconPreview')
conf.set_quoted('PACKAGE_VERSION', meson.project_version())
conf.set('BINDIR', bindir)

conf.set_quoted('COMMIT_ID', 'unknown')
git_command = find_program('git', required: false)
if not git_command.found()
  git_run = run_command('git', ['-C', meson.source_root(), 'describe'])
  if git_run.returncode() == 0
    conf.set_quoted('COMMIT_ID', git_run.stdout().strip())
  endif
endif

subdir('data')
subdir('src')
