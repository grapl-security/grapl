# Setting up CNI networking

## Warning

Firecracker CNI networking doesn't currently work on chromeos due to the lack of
the sch_netem kernel module.

## Running

The build script currently creates the cni config directory if it doesn't exist
and then copies over the CNI config files.
