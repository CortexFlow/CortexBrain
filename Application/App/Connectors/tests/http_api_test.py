from flask import Flask, request, jsonify

app = Flask(__name__)

# Dati temporanei per memorizzare il numero
saved_num = None

# Metodo GET e POST sull'URL di base
@app.route('/', methods=['GET', 'POST'])
def numero():
    global saved_num

    if request.method == 'GET':
        if saved_num is not None:
            return jsonify({'numero': saved_num}), 200
        else:
            return jsonify({'messaggio': 'Nessun numero inviato ancora'}), 404

    elif request.method == 'POST':
        data = request.get_json()  # Ottenere il JSON dal corpo della richiesta
        if 'numero' in data:
            try:
                saved_num = int(data['numero'])  # Convertire il numero a intero
                return jsonify({'messaggio': 'Numero salvato con successo'}), 200
            except ValueError:
                return jsonify({'errore': 'Il valore inviato non è un numero valido'}), 400
        else:
            return jsonify({'errore': 'Nessun numero trovato nella richiesta'}), 400

if __name__ == '__main__':
    app.run(debug=True)
