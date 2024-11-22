#!/bin/bash

CHECK_LIBS="../../../checkLibs.py"
MQTT_TEST='../../App/Connectors/tests/test_mqtt_connection.py'

# Colori per l'output
RESET="\033[0m"
YELLOW="\033[33m"
GREEN="\033[32m"
RED="\033[31m"
CYAN="\033[36m"

TESTS=("$CHECK_LIBS" "$MQTT_TEST" "$TEST_GLOBAL_VAR")
TEST_NAMES=("Installed Libraries Test" "MQTT Connection Test" "Global Variables Test")

success_count=0
fail_count=0
error_count=0

echo -e "${RESET}=== CortexBrain Troubleshooting System ==="
echo "Initializing the testing process... Please wait."
sleep 2

echo -e "${YELLOW}The following tests will be executed:"
for i in "${!TEST_NAMES[@]}"; do
    echo "$(($i + 1)). ${TEST_NAMES[$i]}"
done

sleep 2
echo -e "${RESET}\nPreparing to start the tests..."
echo "Please be patient as the testing process may take some time."
echo "Tests will start shortly..."
echo

for i in "${!TESTS[@]}"; do
    echo -e "\nStarting test $(($i + 1)): ${TEST_NAMES[$i]}"
    echo "Executing: python3 ${TESTS[$i]}"
    
    python "${TESTS[$i]}"
    exit_code=$?

    if [ $exit_code -eq 0 ]; then
        echo -e "${GREEN}Test $(($i + 1)) (${TEST_NAMES[$i]}) completed successfully.${RESET}"
        ((success_count++))
    else
        echo -e "${RED}Test $(($i + 1)) (${TEST_NAMES[$i]}) failed with exit code $exit_code.${RESET}"
        ((fail_count++))
        ((error_count++))
    fi
done

echo -e "=== TEST SUMMARY ==="
echo -e "${YELLOW}Total tests executed: ${#TESTS[@]}${RESET}"
echo -e "${GREEN}Tests succeeded: $success_count${RESET}"
echo -e "${RED}Tests failed: $fail_count${RESET}"
echo -e "${CYAN}Total errors encountered: $error_count${RESET}"
echo "All tests have been executed. Troubleshooting process completed."
