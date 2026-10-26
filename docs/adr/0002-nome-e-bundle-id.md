# Nome del prodotto Scribroom, comando `scrib`, bundle ID `com.fabiotogna.scribroom`

Il prodotto si chiama **Scribroom**; il comando da terminale resta **`scrib`**. La cartella dati diventa `~/Library/Application Support/Scribroom/` (Linux `$XDG_DATA_HOME/Scribroom/`, Windows `%APPDATA%\Scribroom\`). Il bundle ID dell'app è `com.fabiotogna.scribroom`.

Il nome precedente era "Scrib", e il motivo per cui non è più utilizzabile non è il gusto: la parola è già usata da più prodotti che fanno la stessa cosa. `23rd/Scrib` è un trascrittore on-device open source su Android con la stessa promessa, `Scriba` è un registratore di riunioni per macOS, `Scribet` è un voice-to-text locale per macOS basato su whisper.cpp, `Scrivano` registra e trascrive in locale su Linux, `Scripta` è un notetaker privacy-first. Anche `scrib.app` e `getscrib.app` sono occupati, e una domanda di marchio SCRIB in classe 9 è pendente negli Stati Uniti. Un nome qualunque tratto dalla stessa radice non avrebbe differenziato nulla: la famiglia "scrivere" è dove vivono tutti i concorrenti.

"Scribroom" è un token unico (nessun risultato reale sul web, domini `.com`, `.app`, `.dev`, `.io`, `.ai` liberi) che conserva la parola amata e aggiunge l'idea della stanza. Il doppio nome prodotto/comando non è un compromesso: è la norma per gli strumenti da terminale con un nome commerciale (Docker Desktop → `docker`, VS Code → `code`).

Il bundle ID usa la forma `com.<autore>.<prodotto>` perché non esiste un dominio posseduto: Apple richiede unicità, non la proprietà del dominio. Il dominio sarà comprato prima della pubblicazione ufficiale, e non cambierà il bundle ID: il dominio serve al sito, non all'identità dell'app.

Stato: nome e cartella dati in uso; bundle ID deciso e non ancora scritto in nessun `Info.plist`, perché il `.app` non esiste.

## Considered Options

- **Tenere "Scrib"** con una tagline come differenziatore: costo zero, ma il prodotto resta il quinto risultato quando qualcuno cerca "scrib trascrizione".
- **Rinominare fuori dalla famiglia "scrivere"**: più distintivo, ma richiede un nome inventato da zero e la famiglia è talmente affollata che nemmeno "scrib + qualcosa" aiuta.
- **Bundle ID dal dominio** (`com.scribroom.app`): corretto sulla carta, ma lega un identificativo permanente a un acquisto che può non avvenire o non essere rinnovato.
- **`io.github.fabiotognaa.scribroom`**: gratis e unico, ma lega l'identità dell'app all'account GitHub invece che alla persona.

## Consequences

- La cartella dati cambia nome: i dati esistenti vanno spostati a mano, cosa possibile solo perché non ci sono utenti.
- Il bundle ID non si cambia più dopo la prima release: TCC, preferenze e App Store lo usano come identità dell'app. Cambiarlo prima della pubblicazione non costa nulla.
- La documentazione deve tenere allineati il nome del prodotto e il nome del comando, senza usarli come sinonimi.
- Il rename del repository GitHub mantiene i redirect sui link vecchi, ma i cloni esistenti puntano ancora al vecchio remote.
