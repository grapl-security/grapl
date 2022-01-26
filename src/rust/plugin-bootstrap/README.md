## Plugin Bootstrapping

Plugins execute as systemd services. In order to attain the certificates as well
as the binary for the plugin itself we have a precommit hook in systemd to
execute our init binary.

The init binary itself will just reach out to the plugin bootstrap server to
attain the binary and cert, write those to disk, and then exit.

At that point systemd will launch the plugin itself.

When a certificate or binary updates nomad should handle the re-execution of the
whole plugin group.
