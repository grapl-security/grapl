# Clean up some of the stuff pre-installed on Github Actions boxes 
# so we have >10GB to work with.

echo "Starting space: $(df -h | grep ' /$')"

# Based on https://github.com/actions/virtual-environments/issues/709
# Removes 5GB
sudo rm -rf "/usr/local/share/boost"
sudo rm -rf "$AGENT_TOOLSDIRECTORY"

sudo apt-get clean

# Another thing I've seen was to remove /swapfile, but that's
# a tad aggressive. I'll leave this not about it here, though.
# https://github.com/scikit-hep/pyhf/pull/819#issuecomment-616055763

echo "Ending space: $(df -h | grep ' /$')"