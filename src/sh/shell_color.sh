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

declare -A COLORS=(
    [RED]=31
    [GREEN]=32
    [YELLOW]=33
    [BLUE]=34
    [MAGENTA]=35
    [CYAN]=36
    [WHITE]=37
)

RESET="${CSI}${NORMAL}${END}"

function _bold_color() {
    color="${1}"
    shift
    echo "${CSI}${BOLD};${color}${END}${*}${RESET}"
}

function bright_white() {
    _bold_color "${COLORS[WHITE]}" "${@}"
}

function bright_red() {
    _bold_color "${COLORS[RED]}" "${@}"
}

function bright_green() {
    _bold_color "${COLORS[GREEN]}" "${@}"
}

function bright_yellow() {
    _bold_color "${COLORS[YELLOW]}" "${@}"
}
