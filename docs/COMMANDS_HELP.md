# Comandi e opzioni

Riferimento completo della riga di comando di `scrib`. Elenca
ogni comando e ogni opzione con valore, default e vincoli, così il README può
restare concettuale e rimandarli qui.

## Sintassi generale

I comandi seguenti presuppongono che `scrib` sia installato nel
sistema con:

```text
cargo install --path .
```

È possibile disinstallarlo in ogni momento con:

```text
cargo uninstall scrib
```

In alternativa si possono seguire i comandi di questa sezione senza installare
`scrib`, eseguendo direttamente il binario compilato.

```text
scrib <comando> [opzioni]
```

Il binario esegue un solo comando per invocazione e resta in primo piano fino
alla fine. Su errore stampa `error: <messaggio>` ed esce con codice diverso da
zero.

## Opzioni globali

Disponibili su tutti i comandi.

| Opzione           | Tipo                  | Default | Descrizione                                                                          |
| ----------------- | --------------------- | ------- | ------------------------------------------------------------------------------------ |
| `-v`, `--verbose` | conteggio, ripetibile | 0       | Alza il dettaglio dei log diagnostici su standard error. Ripetibile (`-vv`, `-vvv`). |
| `-h`, `--help`    | flag                  | —       | Mostra l'aiuto del comando e delle sue opzioni.                                      |
| `-V`, `--version` | flag                  | —       | Stampa la versione del programma.                                                    |

`--sessions-dir` non è globale: compare solo sui comandi che creano o
indirizzano una sessione (`start`, `doctor`, `transcribe`, `status`, `pause`,
`resume`, `stop`). `devices` ed `export` non lo espongono.

## Cartella dati, config e SCRIB_DATA_DIR

`scrib` non scrive nella directory da cui lo lanci: sessioni e modelli stanno in
una cartella dati per utente.

```text
~/Library/Application Support/Scribroom/
├── config.json      # opzionale, lo scrive l'utente
├── sessions/        # una cartella per sessione o trascrizione
└── models/          # file GGML di Whisper
```

Quello qui sopra è il percorso macOS; su Linux è `$XDG_DATA_HOME/Scribroom/`, su
Windows `%APPDATA%\Scribroom\`.

`config.json` è opzionale e sposta le due sottocartelle:

```json
{
  "sessions_dir": "/Volumes/esterno/scrib/sessions",
  "models_dir": "~/Documenti/modelli"
}
```

- Le chiavi sono `sessions_dir` e `models_dir`, entrambe opzionali.
- I percorsi devono essere assoluti o iniziare con `~`: un percorso relativo è un
  errore, perché non avrebbe un significato ovvio.
- Un JSON malformato o un file vuoto è un errore esplicito: `scrib` non riparte
  in silenzio con i default.
- Le chiavi sconosciute sono ignorate, così un config più nuovo non blocca una
  versione più vecchia.
- Se `config.json` non esiste non viene creato: valgono i default.

La precedenza è: `--sessions-dir` > `SCRIB_DATA_DIR` > `config.json` > default.
`SCRIB_DATA_DIR` sposta l'intera cartella dati e batte il config, che viene
comunque letto e validato. Serve alla modalità sviluppo: per esempio
`SCRIB_DATA_DIR=. scrib start` tiene `sessions/` e `models/` nella directory
corrente, come faceva il vecchio layout dentro al repo.

## Opzioni condivise

Alcune opzioni sono definite una volta e riusate su più comandi. Le opzioni del
**modello** e della **lingua** valgono per `start`, `transcribe` e `doctor`; le
opzioni **VAD** e di **inferenza** valgono per `start` e `transcribe`.

### Modello e lingua

Il prodotto usa un solo modello: `large-v3-turbo` in Q8_0
(`ggml-large-v3-turbo-q8_0.bin`, ~874 MB), cercato in `models/` dentro la cartella
dati (`~/Library/Application Support/Scribroom/models/` su macOS), non nella
directory da cui lanci il comando. Se il file manca, il primo `start` o
`transcribe` lo scarica, ne verifica lo SHA-256 e lo mette nella cartella; `doctor`
non scarica mai e segnala il file mancante.

| Opzione                 | Tipo         | Default                  | Descrizione                                                                                      |
| ----------------------- | ------------ | ------------------------ | ------------------------------------------------------------------------------------------------ |
| `--model <PERCORSO>`    | percorso     | modello predefinito      | File GGML locale. Se passato non scarica nulla: il file deve esistere.                            |
| `--language <LINGUA>`   | `it` \| `en` | `it`                     | Lingua parlata, valida per tutta la sessione o per l'intero file. Nessun rilevamento automatico. |
| `--asr-prompt <TESTO>`  | testo        | prompt predefinito della lingua | Prompt iniziale per Whisper. `--asr-prompt ""` lo disattiva.                              |

Un GGML diverso si seleziona solo con `--model`, che ha la precedenza sul modello
predefinito. I file noti a `doctor` sono elencati in `KNOWN_MODELS`; un nome non
documentato passa con un avviso, senza verifica di identità.

### Segmentazione vocale (VAD)

Flag per impsotare i timing del riconscimento vocale, di seguito le impostazioni di default:

| Opzione                               | Tipo     | Default | Descrizione                                                                       |
| ------------------------------------- | -------- | ------- | --------------------------------------------------------------------------------- |
| `--vad-threshold <NUMERO>`            | decimale | `0.02`  | Soglia RMS oltre la quale un chunk è voce.                                        |
| `--vad-noise-multiplier <NUMERO>`     | decimale | `4.0`   | Quanto la voce deve superare il rumore di fondo stimato per contare come parlato. |
| `--vad-start-ms <MILLISECONDI>`       | intero   | `60`    | Voce consecutiva necessaria ad aprire un segmento.                                |
| `--vad-end-ms <MILLISECONDI>`         | intero   | `1600`  | Silenzio consecutivo necessario a chiudere un segmento.                           |
| `--vad-pre-roll-ms <MILLISECONDI>`    | intero   | `200`   | Audio tenuto prima della prima voce.                                              |
| `--vad-post-roll-ms <MILLISECONDI>`   | intero   | `200`   | Audio tenuto dopo l'ultima voce.                                                  |
| `--vad-min-speech-ms <MILLISECONDI>`  | intero   | `100`   | Sotto questa durata il segmento è scartato (contato in `status.json`).            |
| `--vad-max-segment-ms <MILLISECONDI>` | intero   | `30000` | Tetto di durata; il parlato più lungo è tagliato sulla pausa più lunga.           |

### Inferenza

L'unico flag per la flash attention è attivato di default; `--asr-context` è
spento di default. Per controllare se l'accuraezza migliora a scapito di un esosità di memorie e tempo vi consiglio di fare delle proveComparative.

| Opzione                               | Tipo     | Default | Descrizione                                                                                               |
| ------------------------------------- | -------- | ------- | --------------------------------------------------------------------------------------------------------- |
| `--asr-flash-attention <true\|false>` | booleano | `true`  | Attiva flash attention di whisper.cpp. Va passato un valore esplicito, es. `--asr-flash-attention false`. |
| `--asr-context`                       | flag     | spento  | Porta la coda del segmento precedente nel prompt iniziale (rischio di ripetere un errore precoce).        |

## Comandi

### `devices`

```text
scrib devices
```

Elenca in JSON i dispositivi audio visibili, con direzione supportata e
configurazione predefinita. Non apre alcuno stream. Nessuna opzione oltre a
quelle globali.

### `doctor`

```text
scrib doctor [opzioni]
```

Verifica i prerequisiti locali senza modificare la configurazione di macOS.
Restituisce un report JSON; se un prerequisito manca, `ready` è `false` e
l'exit code è diverso da zero.

| Opzione                     | Tipo     | Default                 | Descrizione                                                                                            |
| --------------------------- | -------- | ----------------------- | ------------------------------------------------------------------------------------------------------ |
| `--model <PERCORSO>`        | percorso | modello predefinito     | Modello da verificare. Senza flag controlla il `large-v3-turbo` della cartella modelli; non scarica.   |
| `--probe-audio`             | flag     | spento                  | Apre entrambi gli input live per tre secondi e verifica che arrivi segnale, senza creare una sessione. |
| `--microphone <NOME>`       | testo    | input predefinito       | Microfono usato dal probe. Richiede `--probe-audio`.                                                   |
| `--system-audio <NOME>`     | testo    | BlackHole               | Input di sistema usato dal probe. Richiede `--probe-audio`.                                            |
| `--sessions-dir <PERCORSO>` | percorso | `sessions_dir` del config, altrimenti `cartella dati/sessions`              | Directory delle sessioni da verificare.                                                                |

Senza `--probe-audio`, permesso del microfono e routing reale restano non
verificati e il report contiene un avviso.

### `start`

```text
scrib start [opzioni]
```

| Opzione                        | Tipo         | Default            | Descrizione                          |
| ------------------------------ | ------------ | ------------------ | ------------------------------------ |
| `--microphone <NOME>`          | testo        | input predefinito  | Dispositivo del microfono.           |
| `--system-audio <NOME>`        | testo        | BlackHole rilevato | Dispositivo dell'audio di sistema.   |
| `--silence-timeout <MINUTI>`   | intero 1–30  | `15`               | Chiude la sessione dopo N minuti di silenzio consecutivo. |
| `--model <PERCORSO>`           | percorso     | predefinito        | Vedi “Modello e lingua”.             |
| `--language <LINGUA>`          | `it` \| `en` | `it`               | Vedi “Modello e lingua”.             |
| `--asr-prompt <TESTO>`         | testo        | predefinito lingua | Vedi “Modello e lingua”.             |
| `--vad-threshold <NUMERO>`     | decimale     | `0.02`             | Vedi “Segmentazione vocale”.         |
| `--vad-start-ms <MS>`          | intero       | `60`               | Vedi “Segmentazione vocale”.         |
| `--vad-end-ms <MS>`            | intero       | `1600`             | Vedi “Segmentazione vocale”.         |
| `--asr-flash-attention <BOOL>` | booleano     | `true`             | Vedi “Inferenza”.                    |
| `--sessions-dir <PERCORSO>`    | percorso     | `sessions_dir` del config, altrimenti `cartella dati/sessions`         | Directory in cui creare la sessione. |

Con `--silence-timeout` la sessione si chiude da sola dopo N minuti consecutivi di
silenzio. Il conteggio è in tempo audio e non si ferma con `pause`: vale la soglia
`--vad-threshold` sul mix, la chiusura è la stessa di `stop` (stato `completed`) e
l'evento `silence_timeout` in `events.jsonl` ne conserva il motivo. Un valore fuori
da 1–30 o non intero viene rifiutato da clap con un errore in inglese.

### `transcribe`

```text
scrib transcribe [FILE] [opzioni]
```

Trascrive un file audio esistente, senza registrare e senza conservare audio: nella
cartella sessioni nasce una cartella nuova (stesso nome `YY-MM-DD` di `start`) con dentro
solo `<nome>.txt`. Non sovrascrive mai nulla, perché la cartella è nuova; se la
trascrizione fallisce non resta niente sul disco. Elabora un solo file per
invocazione, in un unico thread. Se `FILE` è omesso, il percorso viene richiesto su
un terminale interattivo.

Su un terminale mostra una barra di avanzamento su una riga sola: la percentuale
già trascritta sul totale, non il tempo rimanente. Con l'output rediretto non
stampa nulla. I log di whisper.cpp/ggml vanno su standard error e compaiono solo
con `-v`.

| Argomento/Opzione              | Tipo         | Default                                      | Descrizione                                    |
| ------------------------------ | ------------ | -------------------------------------------- | ---------------------------------------------- |
| `<FILE>` (argomento)           | percorso     | opzionale (richiesto su terminale se omesso) | Input `.wav`, `.mp3`, `.m4a`, `.mp4` o `.aac`. |
| `--model <PERCORSO>`           | percorso     | predefinito                                  | Vedi “Modello e lingua”.                       |
| `--language <LINGUA>`          | `it` \| `en` | `it`                                         | Vedi “Modello e lingua”.                       |
| `--asr-prompt <TESTO>`         | testo        | predefinito lingua                           | Vedi “Modello e lingua”.                       |
| `--vad-threshold <NUMERO>`     | decimale     | `0.02`                                       | Vedi “Segmentazione vocale”.                   |
| `--vad-start-ms <MS>`          | intero       | `60`                                         | Vedi “Segmentazione vocale”.                   |
| `--vad-end-ms <MS>`            | intero       | `1600`                                       | Vedi “Segmentazione vocale”.                   |
| `--asr-flash-attention <BOOL>` | booleano     | `true`                                       | Vedi “Inferenza”.                              |
| `--sessions-dir <PERCORSO>`    | percorso     | `sessions_dir` del config, altrimenti `cartella dati/sessions`                                   | Directory in cui creare la cartella di output. |

I WAV passano dal lettore in memoria, quindi ereditano il limite di 64 MiB e 30
minuti; gli altri formati sono decodificati in streaming e non hanno tale
limite.

## Comandi per una sessione in atto

Di seguito sono riporati i comandi da poter eseguire per monitorare e gestire la sessione di transcrizione attuale. Occorre che venga lanciato un secondo terminale in parallelo con quello della sessione corrente in atto.

### `status`

```text
scrib status [opzioni]
```

Stampa il report JSON della sessione corrente: metadati, stato, durata,
profondità delle code, ritardo rispetto all'ultimo segmento ASR e percorsi dei
file.

| Opzione                     | Tipo     | Default    | Descrizione                                  |
| --------------------------- | -------- | ---------- | -------------------------------------------- |
| `--sessions-dir <PERCORSO>` | percorso | `sessions_dir` del config, altrimenti `cartella dati/sessions` | Directory che contiene la sessione corrente. |

### `pause`

```text
scrib pause [opzioni]
```

Sospende la trascrizione lasciando attiva la registrazione. Chiude il segmento
aperto e non attraversa l'intervallo sospeso. In questo modo la trascrizione subisce un taglio fino al lancio di un eventuale `resume`.

| Opzione                     | Tipo     | Default    | Descrizione                                  |
| --------------------------- | -------- | ---------- | -------------------------------------------- |
| `--sessions-dir <PERCORSO>` | percorso | `sessions_dir` del config, altrimenti `cartella dati/sessions` | Directory che contiene la sessione corrente. |

### `resume`

```text
scrib resume [opzioni]
```

Riprende la trascrizione dopo una `pause`.

| Opzione                     | Tipo     | Default    | Descrizione                                  |
| --------------------------- | -------- | ---------- | -------------------------------------------- |
| `--sessions-dir <PERCORSO>` | percorso | `sessions_dir` del config, altrimenti `cartella dati/sessions` | Directory che contiene la sessione corrente. |

### `stop`

```text
scrib stop [opzioni]
```

Finalizza la sessione attiva, questo è il comando che deve essere lanciato per chiudere la registrazione e completare la crezione della cartella di sessione. Attende il completamento. Su una sessione già
`completed` restituisce lo stato senza rifare il lavoro; su una `failed` i
comandi di controllo sono rifiutati.

| Opzione                     | Tipo     | Default    | Descrizione                                  |
| --------------------------- | -------- | ---------- | -------------------------------------------- |
| `--sessions-dir <PERCORSO>` | percorso | `sessions_dir` del config, altrimenti `cartella dati/sessions` | Directory che contiene la sessione corrente. |

### `export`

```text
scrib export <SESSIONE> [opzioni]
```

Rilegge il file `transcript.jsonl` di una sessione e lo converte in jsonl,
testo, Markdown, SRT o VTT, su console o su file con `-o`, senza modificarlo.
Rifiuta come destinazione qualsiasi file canonico della sessione.

| Argomento/Opzione           | Tipo     | Default         | Descrizione                                                                 |
| --------------------------- | -------- | --------------- | --------------------------------------------------------------------------- |
| `<SESSIONE>` (argomento)    | percorso | richiesto       | Directory della sessione.                                                   |
| `--format <FORMATO>`        | enum     | `markdown`      | `jsonl`, `text`, `markdown`, `srt` o `vtt`.                                 |
| `-o`, `--output <PERCORSO>` | percorso | standard output | File di destinazione. La scrittura è atomica.                               |
| `--allow-partial`           | flag     | spento          | Consente l'export di una sessione non completata (fallita o ancora attiva). |

## Comandi per lo sviluppo locale

Comandi da eseguire dalla radice del repository durante lo sviluppo.

- `cargo fmt --check` — verifica la formattazione senza modificare i file.
- `cargo fmt` — riformatta il codice sorgente.
- `cargo clippy --all-targets -- -D warnings` — esegue il linter su tutto il codice e tratta ogni avviso come errore.
- `cargo test` — esegue la suite di test.
- `cargo build` — compila la build non ottimizzata in `target/debug/`.
- `cargo build --release` — compila la build ottimizzata in `target/release/`.
- `cargo run -- <comando>` — esegue il binario compilato senza installarlo, es. `cargo run -- devices`.
- `cargo install --path .` — installa `scrib` nel sistema.
- `SCRIB_MODEL="$HOME/Library/Application Support/Scribroom/models/ggml-small-q5_1.bin" cargo test real_whisper_model_processes_local_audio -- --ignored` — esegue il test con un modello Whisper reale, ignorato per impostazione predefinita.
- `cargo run --release --bin asr-benchmark -- <opzioni>` — esegue il benchmark ASR (opzioni in `benchmarks/README.md`).

controlli della milestone corrente sono:

cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo build
cargo build --release
La build non ottimizzata finisce in target/debug/, quella ottimizzata in target/release/.

Il test con un modello Whisper reale è ignorato per impostazione predefinita e richiede SCRIB_MODEL:

SCRIB_MODEL="$HOME/Library/Application Support/Scribroom/models/ggml-small-q5_1.bin" \
 cargo test real_whisper_model_processes_local_audio -- --ignored
