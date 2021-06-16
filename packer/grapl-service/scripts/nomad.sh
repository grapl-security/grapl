#!/bin/bash -x

set -e

files=/tmp/files

sudo adduser \
	--home=/var/lib/nomad \
	--shell=/usr/sbin/nologin \
	--system \
	nomad

mkdir -p /var/lib/nomad/

nomad_home=~nomad

function install_file() {
	source_basename="$1"
	source_path="${files}/${source_basename}"
	destination_path="$2"
	destination_permissions="$3"
	destination_user="$4"
	destination_group="$5"

	cp "${source_path}" "${destination_path}"
	chmod "${destination_permissions}" "${destination_path}"
	chown "${destination_user}:${destination_group}" "${destination_path}"
}

function install_files() {
	while read l
	do
		install_file $l
	done
}

install_files <<END
nomad          /usr/bin/nomad                     755  root   root
nomad.service  /etc/systemd/system/nomad.service  644  root   root
nomad.conf     ${nomad_home}/nomad.conf           644  nomad  nfsnobody
END

systemctl enable nomad
