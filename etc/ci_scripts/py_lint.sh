#!/bin/bash
set -eu

##########
# A bunch of overhead to just get the directories right
# from https://stackoverflow.com/a/246128
THIS_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"
GRAPL_REPO_ROOT=$(realpath "$THIS_DIR/../..")


##########
# Deal with flags
show_help() {
cat << EOF
Usage:
    -h                 display this help and exit
    -c / --check-only  Don't fix anything, simply check and exit if something failed.
EOF
}

GRAPL_LINT_DIR="$HOME/.venvs/grapl_lint"
FLAG__CHECK_ONLY=

set +u  # need to accept an unbound $1 in this case
while :; do
    case $1 in
        -h|-\?|--help)
            show_help
            exit
            ;;
        -c|--check-only)
            FLAG__CHECK_ONLY=true
            ;;
        -?*)
            printf 'WARN: Unknown option (ignored): %s\n' "$1" >&2
            ;;
        *)               # Default case: No more options, so break out of the loop.
            break
    esac

    shift
done
set -u

##########
# Make the virtualenv if it doesn't exist
if [ ! -d $GRAPL_LINT_DIR ]
then
    echo ">> Making linter virtualenv"
    (
        python3 -mvenv "$GRAPL_LINT_DIR"
        source "$GRAPL_LINT_DIR/bin/activate"
        python -mpip install --upgrade pip
        pip install "black==20.8b1" "isort==5.6.4"
    )
fi


##########
echo ">> Running isort"
(
    source $GRAPL_LINT_DIR/bin/activate
    cd $GRAPL_REPO_ROOT/src/python

    if [ $FLAG__CHECK_ONLY ]
    then
        isort --diff --check-only .
    else
        isort .
    fi
)

##########
echo ">> Running black"
(
    source $GRAPL_LINT_DIR/bin/activate
    cd $GRAPL_REPO_ROOT

    if [ $FLAG__CHECK_ONLY ]
    then
        black --check .
    else
        black .
    fi
)