## Plugin Bootstrapping

Plugin execution is broken up into two systemd services 'init' and 'plugin'.

'init' executes once. It reaches out to the plugin-bootstrap service and retrieves
the plugin binary and client certificate artifacts.

It writes them off to the disk and then exits.

The 'plugin' service depends on the 'init' service and won't execute until 'init' has completed.

The plugin service manages the execution of the plugin itself.
