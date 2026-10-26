# Personal Transcriber

Applicazione locale per registrare riunioni, trascriverle e conservare il risultato sul disco dell'utente. Il contesto copre una sola area: la cattura audio, la trascrizione e gli artefatti che ne derivano.

## Language

### Sessioni e trascrizioni

**Sessione**:
Una registrazione live completa, con audio, eventi, stato e trascrizione, raccolta in una propria cartella.
_Avoid_: recording, run, job

**Sessione live**:
Una sessione che sta registrando e trascrivendo mentre la riunione è in corso.
_Avoid_: diretta, sessione realtime

**Trascrizione da file**:
La conversione in testo di un file audio già esistente. Non cattura audio e non produce una sessione.
_Avoid_: trascrizione offline, batch

**Segmento vocale**:
La porzione di audio, delimitata dal rilevamento di voce, che contiene parlato.
_Avoid_: chunk, blocco

**Segmento trascritto**:
Una riga della trascrizione: il testo di un segmento vocale con il suo intervallo temporale e i metadati di inferenza.
_Avoid_: riga, record

**Trascrizione canonica**:
La trascrizione completa di una sessione, fonte di verità da cui derivano tutti gli export.
_Avoid_: transcript, output, JSONL

**Export**:
Una rappresentazione leggibile della trascrizione canonica, in uno dei formati supportati (markdown, testo, SRT, VTT, JSONL).
_Avoid_: output, report, conversione

### Artefatti su disco

**Cartella di sessione**:
La cartella che contiene gli artefatti di una sessione: audio, metadati, eventi, stato e trascrizione canonica.
_Avoid_: cartella di output, run directory

**Cartella di trascrizione**:
La cartella che contiene solo il testo prodotto da una trascrizione da file. Non è una sessione e nessun comando la scopre.
_Avoid_: sessione inerte, sessione vuota

**Sessione corrente**:
La sessione a cui si riferiscono `status`, `pause`, `resume` e `stop`, indicata da un riferimento nella cartella sessioni.
_Avoid_: sessione attiva, ultima sessione

**Evento di sessione**:
Un fatto registrato durante una sessione: avvio, pausa, ripresa, chiusura, interruzione per silenzio o errore.
_Avoid_: log, messaggio

**Stato della sessione**:
La condizione corrente di una sessione: in avvio, in corso, trascrizione in pausa, in chiusura, completata o fallita.
_Avoid_: fase, status

**Azione di controllo**:
Una richiesta di pausa, ripresa o chiusura inviata alla sessione corrente, applicata una sola volta.
_Avoid_: comando, segnale

**Interruzione per silenzio**:
La chiusura automatica di una sessione live dopo un periodo continuo senza voce.
_Avoid_: watchdog, timeout di inattività

### Modelli

**Modello**:
Il file di pesi Whisper usato per produrre una trascrizione.
_Avoid_: pesi, AI, engine

**Preset**:
Il nome con cui la riga di comando indica un modello documentato, per esempio `large-v3-turbo`.
_Avoid_: alias, scorciatoia

**Identità del modello**:
Il nome e l'impronta SHA-256 registrati con una trascrizione, così si sa con quale modello è stata prodotta.
_Avoid_: versione, hash

### Dati e percorsi

**Cartella dati**:
La radice dell'applicazione, che contiene la cartella sessioni, i modelli e il config.
_Avoid_: data dir, home dell'app

**Cartella sessioni**:
La cartella dentro la cartella dati che raccoglie le cartelle di sessione e di trascrizione.
_Avoid_: cartella di output, archivio

**Config**:
Il file JSON scritto dall'utente nella cartella dati per spostare sessioni o modelli. È opzionale.
_Avoid_: impostazioni, preferenze, settings
