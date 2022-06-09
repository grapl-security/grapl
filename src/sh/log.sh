#!/usr/bin/env bash

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

# Logging
########################################################################

function log() {
    echo -e "${@}" >&2
}

function info() {
    log "$(bright_green INFO):" "${@}"
}

function error() {
    log "$(bright_red ERROR):" "${@}"
}

# Logs an error message and then exits the program.
#
# Exits with code `1` by default, but this can be overridden with the
# `EXIT_CODE` variable, e.g.:
#
#     EXIT_CODE=42 fatal "Aaaaaauuuuuuggggghhhhhhh!"
function fatal() {
    log "$(bright_red FATAL):" "${@}"

    if [ -n "${EXIT_CODE}" ]; then
        exit "${EXIT_CODE}"
    else
        exit 1
    fi
}

function log_and_run() {
    log ❯❯ "$(bright_white "$(printf "%q " "${@}")")"
    "$@"
}
