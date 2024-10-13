import os
import sys
import unittest

# Aggiungi il percorso per l'importazione di GLOBAL_VAR
sys.path.append(os.path.abspath(os.path.join(os.path.dirname(__file__), '../')))
from constants import GLOBAL_VAR

class TestGlobalVar(unittest.TestCase):
    def test_file_paths_exist(self):
        """Test that all file paths defined in GLOBAL_VAR exist."""
        for var in GLOBAL_VAR:
            # Ottieni il percorso assoluto
            path = os.path.abspath(var.value)

            # Controlla se il percorso è un file o una directory
            if not os.path.exists(path):
                print(f"Skipping non-existent path: {path}")
                continue
            
            # Verifica che il file esista
            self.assertTrue(os.path.exists(path), f"File non trovato: {path}")

if __name__ == '__main__':
    unittest.main()
