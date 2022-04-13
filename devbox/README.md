# Devbox

While most Grapl development can take place on our company-provided laptops,
some tasks - particularly when we get into topics like mounting or kernel
modules - require a real Linux box and not just a Chrome OS Crostini container.

The Devbox suite of scripts provisions an EC2 box for this purpose.

## Setup

All of these instructions assume you're in our `$GRAPL_ROOT`

### Provision a box on EC2

- Make sure you're logged in to AWS SSO.
- Run `./devbox/provision.sh`.
- (There is a minor race condition in provision.sh, you may need to run it
  twice. Don't worry, it's idempotent.)

## Usage

### Start a shell session

`./devbox/ssh.sh`

### Sync local files to EC2 (one-way)

`./devbox/devbox-sync.sh`

### Port forwarding

Forward ports from the EC2 machine to your local machine.

- `FORWARD_PORT=1234 ./devbox/ssh.sh`
- `FORWARD_PORT=4646 ./devbox/ssh.sh`

### "Devbox-Do"

This script will sync your files and then execute a command remotely in the
correct directory.

- `./devbox/devbox-do.sh KEEP_TEST_ENV=1 make test-e2e`

## Recommendations

- Add your Devbox-specific SSH key to your SSH agent so you don't need to keep
  typing in the pasword. (I personally like
  [Funtoo Keychain](https://www.funtoo.org/Funtoo:Keychain) to manage this.) You
  can find your SSH key in `~/.grapl_devbox`
- Make `devbox-do` easy to access; I have the following alias setup:

```
alias dbd=/home/wimax/src/repos/grapl/devbox/devbox-do.sh
```
