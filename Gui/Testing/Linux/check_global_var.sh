#!/bin/bash

# Colori per l'output
RESET="\033[0m"
GREEN="\033[32m"
RED="\033[31m"

TEST_GLOBAL_VAR='../../App/Globals/tests/test_global_var.py'

echo "Executing: python3 $TEST_GLOBAL_VAR"
python3 "$TEST_GLOBAL_VAR"
exit_code=$?

exit $exit_code
