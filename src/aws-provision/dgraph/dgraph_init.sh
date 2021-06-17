#!/bin/bash

# create LUKS key and encrypt the drive
echo "encrypting nvme0n1"
sudo bash -c 'head -c 256 /dev/urandom > /root/luks_key'
sudo cryptsetup -v -q luksFormat /dev/nvme0n1 /root/luks_key
sudo bash -c 'echo -e "dgraph\tUUID=$(lsblk -o +UUID | grep nvme0n1 | rev | cut -d" " -f1 | rev)\t/root/luks_key\tnofail" > /etc/crypttab'
sudo systemctl daemon-reload
systemctl start systemd-cryptsetup@dgraph.service
echo "encrypted nvme0n1"

# set up the /dgraph partition
echo "creating /dgraph partition"
sleep 5
sudo mkfs.xfs /dev/mapper/dgraph
sudo mkdir /dgraph
sudo bash -c 'echo -e "/dev/mapper/dgraph\t/dgraph\txfs\tdefaults,nofail\t0\t2" >> /etc/fstab'
sudo mount /dgraph
echo "created /dgraph partition"
