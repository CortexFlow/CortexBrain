import json
import gzip
import time

class DataEncoder:
    def compressJson(data, output_file):
        """
        Args:
        data (dict): The Python dictionary to compress.
        output_file (str): The path to the compressed output file.
        """
        # Convert the dictionary to a minified JSON string
        json_minified = json.dumps(data, separators=(',', ':'))
        
        with gzip.open(output_file, 'wt', encoding='utf-8') as f:
            f.write(json_minified)
            

    def decompressJson(input_file):
        """
        Decompresses a JSON file compressed with gzip and returns the content as a Python dictionary.
        
        Args:
        input_file (str): The path to the compressed input file.
        
        Returns:
        dict: The JSON content of the file decompressed as a Python dictionary.
        """
        with gzip.open(input_file, 'rt', encoding='utf-8') as f:
            data = f.read()
        
        json_data = json.loads(data)
        return json_data

    def JsonMinify(data):
        # Convert the dictionary to a minified JSON string
        json_minified = json.dumps(data, separators=(',', ':'))
        return json_minified
        
        
    def testEncoding(jsonPayload):
        """
        Tests the JsonMinify function by measuring execution time and message size.
        
        Args:
        jsonPayload (dict): The Python dictionary to minify.
        """
        # Original JSON encoding
        start_time = time.time()
        json_data = json.dumps(jsonPayload)  # Original JSON
        json_encode_time = time.time() - start_time
        
        # Minified JSON encoding
        start_time = time.time()
        minified_json = DataEncoder.JsonMinify(jsonPayload)  # Minified JSON
        json_minify_time = time.time() - start_time
        
        # Calculate message sizes
        json_original_size = len(json_data.encode('utf-8'))
        json_minified_size = len(minified_json.encode('utf-8'))
        
        # Convert to megabytes (1 MB = 1,048,576 bytes)
        json_original_size_mb = json_original_size / 1_048_576
        json_minified_size_mb = json_minified_size / 1_048_576
        
        # Print results
        print("Size of the original JSON message:", f"{json_original_size_mb:.6f} MB")
        print("Size of the minified JSON message:", f"{json_minified_size_mb:.6f} MB")
        print(f"JSON encoding time: {json_encode_time:.6E} seconds")
        print(f"JSON minification time: {json_minify_time:.6E} seconds")
        print("\n")
        
        return 0


if __name__ == "__main__":
    # Run the test
    jsonData = {
        "message": "test message",
    }
    DataEncoder.testEncoding(jsonData)
