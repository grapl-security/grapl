# plugin-bootstrap

This crate defines two binary targets:

1. plugin-bootstrap-service - hosts a grpc API that manages plugin artifacts
2. plugin-bootstrap-init - retrieves artifacts from plugin-bootstrap-service and
   places them in the operating system

## Plugin Bootstrapping

Plugin execution is broken up into two systemd services 'plugin-bootstrap-init'
and 'plugin'.

'plugin-bootstrap-init' executes once. It reaches out to the
plugin-bootstrap-service and retrieves the plugin binary and client certificate
artifacts.

It writes them off to the disk and then exits.

The 'plugin' service depends on the 'plugin-bootstrap-init' service and won't
execute until 'plugin-bootstrap-init' has completed.

The plugin service manages the execution of the plugin itself.

### Notes

The following paths are required for the init process to function properly.

```
/usr/local/bin/
/etc/ssl/private/
/etc/systemd/system/plugin.service.d/
```
