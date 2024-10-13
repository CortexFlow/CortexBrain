#!/bin/bash

# Colori per l'output
RESET="\033[0m"
GREEN="\033[32m"
RED="\033[31m"

MQTT_TEST='../../App/Connectors/tests/test_mqtt_connection.py'

echo "Executing: python3 $MQTT_TEST"
python3 "$MQTT_TEST"
exit_code=$?

exit $exit_code
