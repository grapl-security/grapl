#!/usr/bin/env bash

set -euo pipefail

# ANSI Formatting Codes
########################################################################
# Because who wants to remember all those fiddly details?

# CSI = "Control Sequence Introducer"
CSI="\e["
END=m

NORMAL=0
BOLD=1

RED=31
WHITE=37

RESET="${CSI}${NORMAL}${END}"

function _bold_color() {
    color="${1}"
    shift
    echo "${CSI}${BOLD};${color}${END}${*}${RESET}"
}

function bright_white() {
    _bold_color "${WHITE}" "${@}"
}

function bright_red() {
    _bold_color "${RED}" "${@}"
}
