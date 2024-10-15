#!/bin/bash

# Colori per l'output
RESET="\033[0m"
GREEN="\033[32m"
RED="\033[31m"

CHECK_LIBS="../../../checkLibs.py"

echo "Executing: python3 $CHECK_LIBS"
python3 "$CHECK_LIBS"
exit_code=$?

exit $exit_code
