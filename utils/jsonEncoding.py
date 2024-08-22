import json
import gzip
import time

def compressJson(data, output_file):
    """
    Args:
    data (dict): Il dizionario Python da comprimere.
    output_file (str): Il percorso del file di output compresso.
    """
    # Converte il dizionario in una stringa JSON minificata
    json_minified = json.dumps(data, separators=(',', ':'))
    
    with gzip.open(output_file, 'wt', encoding='utf-8') as f:
        f.write(json_minified)
        

def decompressJson(input_file):
    """
    Decomprime un file JSON compresso con gzip e restituisce il contenuto come dizionario Python.
    Args:
    input_file (str): Il percorso del file compresso di input.
    Returns:
    dict: Il contenuto JSON del file decomprimendo come dizionario Python.
    """
    with gzip.open(input_file, 'rt', encoding='utf-8') as f:
        data = f.read()
    
    json_data = json.loads(data)
    return json_data

def JsonMinify(data):
    # Converte il dizionario in una stringa JSON minificata
    JsonMinified = json.dumps(data, separators=(',', ':'))
    return JsonMinified
    
    
def testEncoding(JsonPayload):
    """
    Testa la funzione JsonMinify misurando il tempo di esecuzione e il peso del messaggio.
    Args:
    JsonPayload (dict): Il dizionario Python da minificare.
    """
    # Codifica JSON originale
    start_time = time.time()
    json_data = json.dumps(JsonPayload)  # JSON originale
    json_encode_time = time.time() - start_time
    
    # Codifica JSON minificato
    start_time = time.time()
    minified_json = JsonMinify(JsonPayload)  # JSON minificato
    json_minify_time = time.time() - start_time
    
    # Calcolo del peso dei messaggi
    json_original_size = len(json_data.encode('utf-8'))
    json_minified_size = len(minified_json.encode('utf-8'))
    
    # Conversione in megabyte (1 MB = 1,048,576 bytes)
    json_original_size_mb = json_original_size / 1_048_576
    json_minified_size_mb = json_minified_size / 1_048_576
    
    # Stampa dei risultati
    print("Peso del messaggio JSON originale:", f"{json_original_size_mb:.6f} MB")
    print("Peso del messaggio JSON minificato:", f"{json_minified_size_mb:.6f} MB")
    print(f"Tempo di codifica JSON: {json_encode_time:.6E} seconds")
    print(f"Tempo di minificazione JSON: {json_minify_time:.6E} seconds")
    print("\n")
    
    return 0

if __name__=="__main__":
    # Esegui il test
    JsonData = {
        "message": "test message",
    }
    testEncoding(JsonData)
