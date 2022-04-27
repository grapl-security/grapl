# Summary

This directory includes a dockerfile to create a patched version of scylladb so
it will run in chromeos. Without it, scylladb
[crashes with a SIGFPE error from hwloc](https://github.com/scylladb/scylla/issues/10439).
This is due to a
[chromeos bug](https://bugs.chromium.org/p/chromium/issues/detail?id=1304418).
Fortunately, hwloc has released a
[patch](https://github.com/open-mpi/hwloc/commit/33b555b5a1c8339daeb4215bad57430b57a6b33f)
in 2.7.1.

## Rebuild the libhwloc.so file

Go to https://www.open-mpi.org/software/hwloc/v2.7/ and get the latest release.
Replace `${HWLOC_VERSION}` below with the actual release version

```shell
export HWLOC_VERSION="2.7.1"
curl --proto "=https" \
    --tlsv1.2 \
    --location \
    --output /tmp/hwloc-${HWLOC_VERSION}.tar.gz \
"https://download.open-mpi.org/release/hwloc/v2.7/hwloc-${HWLOC_VERSION}.tar.gz"
cd /tmp
tar -vxzf hwloc-${HWLOC_VERSION}.tar.gz
cd hwloc-${HWLOC_VERSION}
./configure
make
make install
```

The .so file will be created in `/usr/local/lib/` and will typically be named
something similar to `libhwloc.so.15.5.3`

Currently these are uploaded to cloudsmith as raw files
