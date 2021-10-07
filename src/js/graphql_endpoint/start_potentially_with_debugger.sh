#!/bin/bash

SUBSTRING='graphql_endpoint'
if [[ "$DEBUG_SERVICES" == *"$SUBSTRING"* ]]; then
    echo "Starting with debugger"
    node --inspect="0.0.0.0:${VSC_DEBUGGER_PORT_FOR_GRAPHQL_ENDPOINT}" server.js
else
    echo "Starting sans debugger"
    node server.js
fi
