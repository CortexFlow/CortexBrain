def CheckLib(libraries):
    not_installed = []

    special_imports = {
        'PyQtWebEngine': 'PyQt5.QtWebEngineWidgets',  # Nome del modulo effettivo per importare PyQtWebEngine
    }

    for library in libraries:
        # Usa un nome di importazione speciale se presente
        module_to_import = special_imports.get(library, library)
        try:
            __import__(module_to_import)
            print(f"The library '{library}' is installed.")
        except ImportError:
            print(f"The library '{library}' is NOT installed.")
            not_installed.append(library)

    if not_installed:
        print("\nThe following libraries are not installed. Please install the following libraries:")
        for lib in not_installed:
            print(f"- {lib}")
    else:
        print("\nAll libraries are installed.")


if __name__ == "__main__":
    # Example usage:
    libraries_to_check = ['numpy', 'pandas', 'matplotlib', 'prophet', 'requests', 'asyncio',
                          'aiohttp', 'networkx', 'folium', 'pydot',
                          'geopy', 'simpy', 'pymongo', 'snowflake', 'PyQt5', 'PyQtWebEngine', 'osmnx', 'mesa','openpyxl']
    CheckLib(libraries_to_check)



if __name__=="__main__":
    # Example usage:
    libraries_to_check = ['numpy', 'pandas', 'matplotlib', 'prophet', 'requests', 'asyncio',
                        'aiohttp', 'networkx', 'folium', 'pydot',
                        'geopy', 'simpy','pymongo','snowflake','PyQt5','PyQtWebEngine','osmnx','mesa']
    CheckLib(libraries_to_check)
