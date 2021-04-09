#!/bin/bash

checkPrettierInstalled() {
    npm list --depth 1 --global prettier > /dev/null 2>&1
    not_installed=$?
    if [ "$not_installed" -ne "0" ]; then
        echo "Installing prettier" && npm install -g prettier;
    fi
}

prettier --config grapl-cdk/.prettierrc.toml .
