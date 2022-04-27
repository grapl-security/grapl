FROM scylladb/scylla

# This downloads a prebuild static lib for hwloc 2.7.1 which includes a chromeos fix, and replaces the dynamically
# linked version that scylla uses. See the README for more info

RUN curl --proto "=https" \
    --tlsv1.2 \
    --location \
    --output /usr/local/lib/libhwloc.so.15.5.3 \
    "https://dl.cloudsmith.io/public/grapl/thomas/raw/versions/15.5.3/libhwloc.so.15.5.3" && \
    rm /opt/scylladb/libreloc/libhwloc.so.15 && \
    ln -sf /usr/local/lib/libhwloc.so.15.5.3 /opt/scylladb/libreloc/libhwloc.so.15

