# La sessione gira nell'app: il core Rust è linkato, non un processo figlio

L'app desktop linka il crate `scrib` e chiama `session::run()` su un thread dedicato: la sessione gira nello stesso processo della UI. Il binario `scrib` resta nel bundle per chi usa il terminale, ma non è il canale tra app e sessione. L'alternativa era il sidecar: l'app fork-a il binario, ne legge i file e lo comanda via `control.json`.

Il filesystem era già il contratto della sessione (`status.json`, `transcript.jsonl`, `control.json`), quindi il sidecar non avrebbe aggiunto un protocollo: solo un processo da supervisionare e un argv da tenere allineato a `cli.rs`. Non avrebbe comprato latenza, perché i segmenti arrivano quando Whisper chiude una frase, non ogni 250 ms, e la UI legge comunque i file. E non avrebbe salvato la registrazione da un abort di ggml: muore il processo che esegue il loop, figlio o app che sia. Restava l'isolamento della UI, pagato con adozione delle sessioni orfane, liveness del lock e due binari da firmare. Il prezzo accettato è esplicito: se la UI muore, la sessione muore con lei.

## Consequences

- Una sola sessione per volta resta vera per tutti: `.runner.lock` è tenuto dal processo dell'app, quindi uno `scrib start` dal terminale fallisce con `an active session already exists`.
- Il controllo resta su file: app e CLI chiamano `control::request()`, quindi `scrib pause/resume/stop` funziona su una sessione avviata dall'app e viceversa.
- La UI osserva la sessione da `status.json` e `transcript.jsonl`; non c'è un canale in più verso la UI. Se un giorno servirà più granularità, il seam `SegmentSink` esiste già.
- Chiusura della finestra e ⌘Q passano dallo stesso handler: con una sessione attiva chiedono conferma e fanno stop pulito. Nessun orfano, nessuna adozione al riavvio.
- Un crash dell'app lascia lo stato "in corso": al successivo avvio `recover_stale_session` marca la sessione failed e ripara i JSONL. L'header WAV non viene finalizzato (`WavWriter::finalize` gira solo allo stop), quindi la recovery deve ricostruirlo o l'audio resta illeggibile.
- `StartOptions` va costruito in un punto solo, condiviso da app e `cli.rs`: due assemblaggi divergono alla prima opzione nuova.
- `list_sessions` sta nella lib e applica la regola già usata da `export`: uno `schema_version` più nuovo del supportato è un errore esplicito, non una lettura parziale.

Stato: deciso prima della UI, nessun codice lo implementa ancora; il punto 8 dei TODO dipende da questa forma.

## Considered Options

- **Sidecar** (l'app lancia `scrib start` e ne legge i file): isola la UI e lascia registrare dopo la chiusura della finestra, ma trasforma l'argv in un'API non tipizzata, aggiunge supervisione e adozione degli orfani, e non salva la registrazione da un abort del motore.
- **Helper nel bundle con socket**: eventi più fini e progresso (per esempio il download del modello) senza polling, ma un protocollo da versionare per un guadagno che oggi non esiste; un `--json-stream` su stdout coprirebbe lo stesso bisogno senza cambiare questa decisione.
- **GUI che legge le cartelle da sola**: duplica `paths.rs` (cartella dati, `SCRIB_DATA_DIR`, `config.json`) e mostra una lista vuota a chi ha spostato `sessions_dir` nel config.
