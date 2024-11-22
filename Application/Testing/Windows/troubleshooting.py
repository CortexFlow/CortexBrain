import os
import time
from colorama import Fore,Style

CHECK_LIBS = "../../../checkLibs.py"
MQTT_TEST = '../../App/Connectors/tests/test_mqtt_connection.py'

if __name__ == "__main__":
    TESTS = [CHECK_LIBS,MQTT_TEST]
    TEST_NAMES = ['Installed Libraries Test','MQTT Connection Test', 'Global Variables Test']
    
    success_count = 0
    fail_count = 0
    error_count = 0

    print(Style.RESET_ALL+'=== CortexBrain Troubleshooting System ===')
    print('Initializing the testing process... Please wait.')
    time.sleep(2)
    
    print(Fore.YELLOW+'The following tests will be executed:')
    for i, test in enumerate(TEST_NAMES):
        print(f'{i+1}. {test}')
    
    time.sleep(2)
    print(Style.RESET_ALL+'\nPreparing to start the tests...')
    print('Please be patient as the testing process may take some time.')
    print('Tests will start shortly...\n')
    
    for i, test in enumerate(TESTS):
        print(f'\nStarting test {i+1}: {TEST_NAMES[i]}')
        print(f'Executing: python {test}')
        
        exit_code = os.system(f"python {test}")
        
        if exit_code == 0:
            print(f'Test {i+1} ({TEST_NAMES[i]}) completed successfully.')
            success_count += 1
        else:
            print(f'Test {i+1} ({TEST_NAMES[i]}) failed with exit code {exit_code}.')
            fail_count += 1
            error_count += 1
        
    
    print('=== TEST SUMMARY ===')
    print(Fore.YELLOW+f'Total tests executed: {len(TESTS)}'+Style.RESET_ALL)
    print(Fore.GREEN+f'Tests succeeded: {success_count}'+Style.RESET_ALL)
    print(Fore.RED+f'Tests failed: {fail_count}'+Style.RESET_ALL)
    print(Fore.CYAN+f'Total errors encountered: {error_count}')
    print('All tests have been executed. Troubleshooting process completed.'+Style.RESET_ALL)
