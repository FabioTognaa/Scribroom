# Cartella dati in Application Support, non nella directory corrente

I dati dell'app (sessioni, modelli, config) stanno in `~/Library/Application Support/Scribroom/`, su Linux in `$XDG_DATA_HOME/Scribroom/`, su Windows in `%APPDATA%\Scribroom\`. In precedenza l'app cercava `models/` e `sessions/` nella directory corrente: funzionava solo dentro al repo, mentre un binario installato deve funzionare da qualsiasi directory e non deve scrivere in cartelle scelte dall'utente (o protette da macOS).

L'ordine di risoluzione è flag > variabile `SCRIB_DATA_DIR` > config > default assoluto. La variabile d'ambiente esiste per la modalità sviluppo, non per l'utente finale, e per questo batte il config. Il config accetta solo percorsi assoluti o con `~`, perché un percorso relativo non ha un significato ovvio.

Stato: flag, variabile d'ambiente, config e default sono in uso. Il nome della cartella dentro la radice è cambiato da `Scrib` a `Scribroom`: la decisione sul nome è nella [0002](0002-nome-e-bundle-id.md).

## Considered Options

- **Directory corrente** (stato precedente): nessuna configurazione, ma l'app scrive dove capita e non trova più i file dopo l'installazione.
- **`~/Documents`**: visibile all'utente, ma i dati dell'app non sono documenti e macOS chiede un permesso di accesso.
- **`~/Library/Caches`**: adatto ai dati rigenerabili, ma un modello da centinaia di MB verrebbe riscaricato a ogni pulizia della cache.

## Consequences

- Spostare la radice in seguito richiede una migrazione: la decisione va presa prima dei primi utenti.
- La GUI futura e la CLI devono usare la stessa funzione di risoluzione, altrimenti leggono cartelle diverse.
- I file già presenti in `models/` e `sessions/` dentro al repo non vengono trovati: vanno spostati o indicati nel config.
