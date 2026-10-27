# Installazione per macOS Apple Silicon

## Requisiti

- macOS su Apple Silicon. I binari pubblicati sono `arm64`; su Intel la build da
  sorgente è l'unica strada.
- BlackHole, **solo per registrare una call live** (`start`). Per trascrivere un
  file già esistente (`transcribe`) non serve.

## 1. Installazione con un comando

```sh
curl -fsSL https://github.com/FabioTognaa/Scribroom/releases/latest/download/install.sh | sh
```

Lo script scarica il tarball dell'ultima release, ne verifica lo SHA-256 e installa
`scrib` in `$(brew --prefix)/bin` se Homebrew è presente, altrimenti in
`/usr/local/bin` (chiede la password solo se la cartella non è scrivibile). Non
modifica il PATH e non scarica il modello. Rilanciare lo stesso comando aggiorna
un'installazione esistente.

Verifica:

```sh
scrib --version
```

Se il sistema non è macOS o l'architettura non è `arm64`, lo script si ferma con un
messaggio che rimanda alla build da sorgente.

## 2. Il modello

Il primo `start` o `transcribe` scarica `large-v3-turbo` (Q8_0, ~874 MB) in
`~/Library/Application Support/Scribroom/models/`, ne verifica lo SHA-256 e lo usa.
Su un terminale mostra una barra di avanzamento; con l'output rediretto resta
silenzioso. Se il download si interrompe o l'hash non torna, il file parziale viene
cancellato e il comando esce con un errore.

Per usare un GGML che hai già:

```sh
scrib start --model /percorso/del/modello.bin
```

`--model` ha la precedenza sul modello predefinito e non scarica nulla. Senza rete e
senza `--model`, il comando fallisce spiegando le due opzioni.

## 3. Verifica dei prerequisiti

```sh
scrib doctor
```

Stampa un report JSON: se qualcosa manca, il controllo corrispondente è `fail`,
`ready` è `false` e il comando esce con codice diverso da zero. Il modello risulta
mancante finché non fai il primo `start`; il dettaglio spiega che verrà scaricato.
Per provare anche microfono e audio di sistema servono BlackHole configurato e
`--probe-audio`.

## 4. Prima trascrizione

Per una call live, installa e configura BlackHole seguendo la
[guida macOS](BLACKHOLE_MACOS.md), poi:

```sh
scrib start --language it
```

`scrib status`, `scrib pause`, `scrib resume` e `scrib stop` agiscono sulla sessione
corrente. Per un file già registrato, senza BlackHole:

```sh
scrib transcribe riunione.mp3 --language it
```

## 5. Sviluppo

Con il repository clonato e Rust 1.88+:

```sh
cargo run --release -- start
```

Per installare la build locale invece del binario della release:

```sh
cargo install --path .
```

`SCRIB_DATA_DIR=.` riporta `sessions/` e `models/` nella directory corrente. Il
dettaglio di percorsi, config e precedenza è in
[Comandi e opzioni](COMMANDS_HELP.md#cartella-dati-config-e-scrib_data_dir).
