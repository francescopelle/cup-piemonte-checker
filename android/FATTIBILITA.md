# App Android: analisi di fattibilità

> **Aggiornamento.** Dopo questa analisi sono state prese decisioni diverse su alcuni punti: l'app si scrive in
> Kotlin Multiplatform sopra il motore Rust (non con Tauri) e il codice sta nella cartella `android/`. Vale il
> [piano di sviluppo](PIANO.md); questo documento resta come motivazione e per le verifiche fatte sul codice.

Novembre 2026, sulla versione 0.3.2 dell'app desktop. Obiettivo: un'app per Android distribuita come **APK da
scaricare e installare** (niente Play Store per ora), con le stesse funzioni dell'app per il computer.

## In breve

**Si può fare, riusando quasi tutto.** Il motore (`core/`) compila per Android senza modifiche, Tauri 2 (il
framework dell'app desktop) supporta Android, e l'interfaccia HTML è già pensata per finestre larghe 360 pixel,
cioè quanto un telefono.

**Il punto difficile non è il portale ma il telefono in tasca.** Sul computer i controlli ogni 20 minuti sono un
thread che dorme e si risveglia. Su Android, a schermo spento, il sistema sospende processore e rete e chiude le
app in background. Serve un *servizio in primo piano* con una notifica fissa (l'equivalente Android dell'icona
nella barra di sistema) e un po' di codice Kotlin nostro. Quanto i controlli restano regolari dipende anche dalla
marca del telefono: va misurato con un prototipo su telefoni veri prima di impegnarsi sul resto.

**Cinque pezzi dell'app desktop su Android non esistono** (icona nella barra, avvio con il computer, copia unica,
aggiornamenti automatici, appunti) e vanno sostituiti. **Un sesto, il portachiavi, compila ma su Android non salva
niente, senza dare errore**: lasciato così, l'app perderebbe le ricette a ogni riavvio (dettagli sotto).

**Stima grossolana:** una prima versione completa nelle funzioni (controlli, notifiche, sposta, prenota e
disdici col "Sì", prenotazione automatica) in un APK firmato richiede circa **4-6 settimane di lavoro** di una
persona; con l'aggiornamento automatico dell'APK e la chiave nel Keystore di Android circa **6-8 settimane**.

**Distribuire l'APK oggi in Italia è semplice** (si consente "Installa app sconosciute"). **Dal 2027** Google
estende anche all'Europa l'obbligo di sviluppatori verificati per le app installate fuori dal Play Store: prima di
allora bisogna registrarsi (gratis fino a 20 dispositivi, 25 $ per distribuire a tutti), altrimenti chi installa
dovrà passare dal "flusso avanzato" di Android o da ADB.

## Cosa ho verificato

Prove fatte sul codice di questo repository, non stime.

| Verifica | Esito |
|---|---|
| `cargo check -p cup-core --target aarch64-linux-android` | ✅ **Il motore compila per Android a 64 bit.** reqwest usa rustls (niente OpenSSL da compilare), ring, regex e chrono passano. *Limite della prova*: qui non si poteva scaricare l'NDK di Android, quindi è una verifica dei tipi con header C finti per ring, non una build completa. ring e rustls sono comunque usati di continuo su Android. |
| `cargo check -p cup-desktop --target aarch64-linux-android` | ❌ Non compila, ed è previsto. Primo errore: `arboard` (appunti) non ha una versione Android. Tolto quello: `tauri::tray` e `tauri::menu` (icona nella barra), `tauri_plugin_autostart`, `tauri_plugin_single_instance` e `WebviewWindow::unminimize` esistono solo sul computer. |
| `keyring` 3.6.3 su `target_os = "android"` | ⚠️ **Compila ma usa il portachiavi finto (`mock`) della libreria, che non salva nulla.** In `archivio.rs` la rilettura subito dopo la scrittura riuscirebbe (stesso oggetto in memoria), quindi la chiave sembrerebbe salvata. Al riavvio però se ne crea una nuova, l'archivio risulta "illeggibile" e viene messo da parte: le ricette sparirebbero a ogni riavvio. Su Android il portachiavi va escluso a mano. |
| `tauri-plugin-updater` 2.14.1 | Compila, ma dichiara supporto Android "none": non installa nulla. L'app ha già il caso giusto: senza tipo di installazione (`bundle_type()` è `None`) va in `Modo::NonDisponibile`. |
| `tauri-plugin-notification` 2.7.0 e `tauri-plugin-dialog` 2.8.3 | ✅ Hanno il loro codice Kotlin per Android. |
| Dimensione del motore | `cup-cli` in release (Linux x86_64, `opt-level = "s"`, LTO) è 4,0 MB: la libreria nativa per arm64 sarà simile. Un APK per una sola architettura dovrebbe stare intorno ai 10-15 MB (da misurare nel prototipo). |
| Interfaccia (`desktop/ui/`) | `<meta name="viewport">` già presente, colonna larga al massimo 560 px, finestra minima 360 px, tema scuro. Da cambiare i testi che parlano di "computer" e "barra di sistema" e l'opzione "Avvia con il computer". |

## Cosa si riusa e cosa cambia

| Parte | Sul computer | Su Android | Lavoro |
|---|---|---|---|
| `core/` (motore, ~4.500 righe) | | uguale | nessuno |
| `controlli.rs`, `operazioni.rs`, `archivio.rs`, `registro.rs`, `segnalazioni.rs` (~3.700 righe con le prove) | | uguali, spostati in una crate comune a desktop e Android | basso |
| `main.rs` (comandi per l'interfaccia, avvio) | | i comandi restano; icona, avvio con il computer e copia unica solo per desktop (`#[cfg(desktop)]`); punto d'ingresso in una `lib.rs` (`tauri::mobile_entry_point`) | medio |
| Interfaccia HTML/JS/CSS (~720 righe) | | la stessa, con testi e impostazioni per il telefono | basso |
| Icona nella barra di sistema | `TrayIconBuilder` | notifica fissa del servizio in primo piano, con "Apri" e "Ferma i controlli" | nel lavoro sul background |
| Avvio con il computer | `tauri-plugin-autostart` | all'accensione del telefono (`BOOT_COMPLETED`) si riavvia il servizio | basso |
| Una copia sola | `tauri-plugin-single-instance` | Android ha già un processo per app; da verificare che riaprendo l'app non parta un secondo thread dei controlli (vedi sotto) | basso, da verificare |
| Thread dei controlli | `sleep(5 s)` in un ciclo | servizio in primo piano + sveglia periodica + wake lock solo durante il controllo | **alto: il rischio principale** |
| Chiave delle ricette | portachiavi del sistema (`keyring`) | prima versione: la chiave in un file nella cartella privata dell'app (`Protezione::File`, già esistente) e backup di Android disattivato; poi Android Keystore | basso, poi medio |
| Appunti ("Copia") | `arboard` | `tauri-plugin-clipboard-manager` o `ClipboardManager` di Android, segnando il testo come sensibile (Android 13+ non lo mostra nell'anteprima) | basso |
| Aggiornamenti | `tauri-plugin-updater` | prima versione: avviso "c'è la versione X" con link; poi scaricamento dell'APK, verifica della firma e installatore di sistema | medio |

Il resto vale così com'è: le zone, i giorni da evitare, il limite di un controllo ogni 20 minuti salvato sul
disco, la prenotazione automatica, il salvataggio prima della Conferma, gli esiti incerti, i contatti per il CUP,
il registro senza dati personali e le segnalazioni facoltative.

## Il nodo: controllare ogni 20 minuti con il telefono in tasca

### Cosa fa Android

- **Doze e App Standby.** A schermo spento e telefono fermo, Android sospende processore e rete e rimanda timer e
  lavori in background a brevi "finestre di manutenzione", sempre più rade. Un `thread::sleep` di Rust si ferma
  con il processore.
- **WorkManager** (il modo "standard" per lavori periodici) ha un intervallo minimo di 15 minuti, ma in Doze e
  per le app usate di rado li rimanda anche di ore. Da solo non basta.
- **Servizio in primo piano.** Un'app con un servizio in primo piano, e la sua notifica fissa, non viene
  considerata inattiva: il processo resta vivo e mantiene la rete. Da Android 14 il servizio dichiara un tipo:
  `dataSync` da Android 15 è limitato a 6 ore al giorno, quindi serve `specialUse` (fuori dal Play Store basta
  dichiararlo; sul Play Store va giustificato). Da Android 12 un servizio così non si può avviare dal background,
  salvo eccezioni (accensione del telefono, app esclusa dall'ottimizzazione della batteria): si avvia quando
  l'utente apre l'app o all'accensione, e poi resta acceso.
- **Svegliare il processore ogni 20 minuti**: `AlarmManager.setAndAllowWhileIdle`, una sveglia non esatta che
  scatta anche in Doze (al massimo circa ogni 10-15 minuti, compatibile con 20) e non richiede permessi speciali.
  Alla sveglia si prende un *wake lock* parziale solo per la durata del controllo. Un wake lock tenuto sempre
  consumerebbe la batteria: da evitare.
- **Ottimizzazione della batteria.** L'app chiede all'utente di escluderla
  (`REQUEST_IGNORE_BATTERY_OPTIMIZATIONS`): un dialogo di sistema, un tocco. Fuori dal Play Store è consentito.
- **Produttori.** Xiaomi (HyperOS/MIUI), Huawei, Samsung ("app in sospensione"), Oppo e OnePlus aggiungono
  regole loro e chiudono le app in background più di Android puro. Servono istruzioni per marca nell'app o nella
  pagina di download (il sito dontkillmyapp.com le raccoglie). Resta un rischio che non si elimina del tutto.

**Proposta:** servizio in primo piano `specialUse`, acceso finché c'è almeno una ricetta attiva, con una notifica
silenziosa ("Controllo 2 ricette ogni 20 minuti", "Apri", "Ferma i controlli"); sveglia non esatta ogni 20
minuti; wake lock durante il controllo e durante spostamenti, prenotazioni e disdette; richiesta di esclusione
dall'ottimizzazione della batteria al primo avvio; riavvio del servizio all'accensione.

Se un controllo arriva in ritardo non si rompe niente: si perde qualche data che esce e viene presa subito. Il
limite dei 20 minuti è un minimo, salvato sul disco, e continua a valere.

### Cosa va verificato con un prototipo

Tauri 2 non ha un supporto ufficiale per il lavoro in background su Android. Il codice Rust gira nel processo
dell'app, avviato dall'Activity (la schermata). Le domande aperte:

1. Chiusa l'app (anche togliendola dalle app recenti), con il servizio acceso, i thread Rust continuano?
2. Le notifiche di `tauri-plugin-notification` partono anche senza Activity? Se no, le mostra il codice Kotlin
   del servizio.
3. Riaprendo l'app, Tauri riparte senza creare un secondo `Stato` e un secondo thread dei controlli? Due thread
   bloccherebbero due date per la stessa ricetta: è proprio ciò che sul computer evita la copia unica.
4. Quanto sono regolari i controlli, a schermo spento, su un Pixel, un Samsung e uno Xiaomi?

Esiste un plugin di terze parti, `tauri-plugin-background-service`, che dichiara un servizio che sopravvive alla
chiusura dell'app: da valutare nel prototipo, ma il servizio è poco codice Kotlin e scriverlo noi evita una
dipendenza in un punto delicato.

### Operazioni interrotte

Android può chiudere il processo anche a metà di uno spostamento (memoria, batteria, utente). L'app è già fatta
per questo: salva sul disco che l'operazione sta partendo prima della Conferma e, al riavvio, segna l'esito come
incerto e ferma la ricetta finché l'utente non ha controllato. Su Android in più si tiene il wake lock e il
servizio in primo piano per tutta l'operazione.

## Tre strade

**A. Tauri 2 per Android (consigliata).** Lo stesso Rust, la stessa interfaccia, gli stessi comandi; in più un
piccolo plugin Kotlin nostro per servizio, sveglia, accensione e batteria. È la strada con meno codice nuovo e una
sola interfaccia da mantenere. Il rischio sta nel ciclo di vita (le quattro domande sopra).

**B. App nativa in Kotlin, con il Rust sotto.** Interfaccia riscritta in Kotlin (Jetpack Compose), motore e
logica in Rust chiamati con UniFFI. Pieno controllo di servizio e ciclo di vita, ma una seconda interfaccia da
scrivere e mantenere. È il ripiego se il prototipo di A non regge.

**C. Niente app.** Il **bot Telegram** funziona già sul telefono (ma serve un server e codice fiscale e ricetta
passano dal server di chi lo gestisce). Una **web app (PWA)** non è una strada: da una pagina web il browser
quasi certamente non lascerebbe leggere le risposte del portale CUP (un altro sito), e una pagina web non lavora
in background.

In A e in B il primo passo è lo stesso: portare la logica di `desktop/` in una crate comune. Il lavoro non va
perso se dopo il prototipo si cambia strada.

## Distribuire l'APK

- **Firma.** L'APK si firma con una chiave nostra (keystore). Android installa un aggiornamento solo se è firmato
  con la stessa chiave: perderla vuol dire che gli utenti devono disinstallare l'app, e con essa le ricette, per
  avere la versione nuova. Va nei segreti del repository, con una copia fuori da GitHub. Anche il nome del
  pacchetto (`io.github.ilmondovero.cup`) non si cambia più dopo la prima versione.
- **Installazione.** Si scarica l'APK dalla pagina di download o dalla Release e si consente al browser di
  installare app sconosciute. Play Protect può mostrare un avviso o proporre una scansione. Conviene non chiedere
  i permessi che fanno scattare i blocchi più severi (SMS, accessibilità, lettura delle notifiche): all'app non
  servono. Servono `INTERNET`, `POST_NOTIFICATIONS`, `FOREGROUND_SERVICE` e `FOREGROUND_SERVICE_SPECIAL_USE`,
  `RECEIVE_BOOT_COMPLETED`, `WAKE_LOCK`, `REQUEST_IGNORE_BATTERY_OPTIMIZATIONS` e, solo per l'aggiornamento dentro
  l'app, `REQUEST_INSTALL_PACKAGES`.
- **Versioni di Android.** Tauri 2 richiede Android 7.0 o successivo. Proposta: solo arm64 (tutti i telefoni
  degli ultimi anni); armv7 a 32 bit si aggiunge se qualcuno ha un telefono vecchio.
- **Verifica degli sviluppatori di Google.** Dal 30 settembre 2026, in Brasile, Indonesia, Singapore e Thailandia,
  i telefoni certificati installano solo app di sviluppatori verificati; nel resto del mondo dal 2027 (per l'Europa
  le fonti trovate non danno ancora una data). Le app di sviluppatori non verificati restano installabili con ADB o
  con il "flusso avanzato" (passi in più e un'attesa): troppo per chi non è pratico. Le possibilità:
  - account gratuito a **distribuzione limitata**: fino a 20 dispositivi, senza documento d'identità, per
    famiglia e amici;
  - **registrazione completa** (25 $, con documento): per distribuire a chiunque;
  - più avanti il Play Store, con le sue regole in più: giustificare il servizio `specialUse` e l'esclusione
    dalla batteria, e un test chiuso con almeno 12 tester per 14 giorni per un account personale nuovo.

  Le date vanno ricontrollate sulle pagine di Google prima di decidere: le ho ricavate da risultati di ricerca,
  senza poter aprire le pagine da qui.
- **Backup.** Android copia i dati delle app su Google Drive. Con la chiave in un file, file delle ricette e chiave
  finirebbero nello stesso backup: va disattivato (`android:allowBackup="false"` o regole di esclusione). Codice
  fiscale, ricetta e dati della visita sono dati sanitari: devono restare sul telefono come oggi restano sul
  computer.
- **Build su GitHub Actions.** Un job Android in `app-installer.yml`: le macchine Ubuntu di GitHub hanno Android SDK
  e NDK; servono JDK 17 e il target Rust `aarch64-linux-android`; poi `cargo tauri android build --apk`, firma,
  somma SHA-256 nella release. Come per i computer, prima di caricarlo si prova ad avviarlo, su un emulatore,
  aspettando "avvio completato" nel registro.
- **Aggiornamenti.** Prima versione: l'app legge lo stesso `latest.json` (con una voce in più per Android) e avvisa
  con una notifica e un link. Poi: scarica l'APK, ne verifica la firma con la stessa chiave degli aggiornamenti
  desktop, rispetta le 48 ore e il ritiro, e apre l'installatore di Android. A ogni aggiornamento l'utente
  conferma con un tocco: un'app fuori dal Play Store non si aggiorna in silenzio.

## Piano proposto

| Fase | Cosa | Stima |
|---|---|---|
| 0. Prototipo | Progetto Tauri Android minimo con `cup-core`, APK firmato da GitHub Actions, servizio in primo piano e sveglia, un controllo in sola lettura ogni 20 minuti. 48 ore a schermo spento su 3 telefoni (Pixel, Samsung, Xiaomi), con l'orario di ogni controllo nel registro. **Si va avanti se** l'intervallo tipico resta sotto i 30 minuti, le notifiche arrivano ad app chiusa e riaprendo l'app non parte un secondo thread. Altrimenti strada B. | 3-5 giorni |
| 1. Crate comune | Spostare archivio, controlli, operazioni, registro e segnalazioni da `desktop/` a una crate comune, con le prove che ci sono già. L'app desktop non cambia. | 3-5 giorni |
| 2. Guscio Android | `lib.rs` con il punto d'ingresso mobile, `#[cfg(desktop)]` su icona, avvio e copia unica, plugin Kotlin (servizio, sveglia, accensione, batteria, permesso delle notifiche), chiave in file e backup spento, appunti. | 6-10 giorni |
| 3. Interfaccia | Testi per il telefono, impostazioni Android (notifiche, batteria, istruzioni per marca), tasto Indietro, margini dello schermo, pulsanti grandi abbastanza per il dito. | 2-4 giorni |
| 4. Build e distribuzione | Job Android in GitHub Actions, keystore, APK nella release, pagina di download con istruzioni per l'installazione. | 2-3 giorni |
| 5. Prova sul campo | Versione di prova a pochi utenti, correzioni. | 1-2 settimane di calendario, poco lavoro |
| 6. Dopo la prima versione | Aggiornamento dell'APK dentro l'app (3-5 giorni), chiave nel Keystore di Android (2-3 giorni). | 5-8 giorni |

Le stime sono grossolane: il prototipo serve anche a renderle più precise.

## Rischi

| Rischio | Probabilità | Effetto | Cosa fare |
|---|---|---|---|
| Controlli irregolari in background (Doze, regole dei produttori) | alta su alcune marche | date perse, non danni | servizio in primo piano, sveglia, esclusione dalla batteria, istruzioni per marca; misurarlo nel prototipo |
| Ciclo di vita di Tauri su Android (secondo thread, notifiche senza Activity) | media | due date bloccate, notifiche perse | prototipo; ripiego sulla strada B |
| Portachiavi finto di `keyring` | certa se non gestito | ricette perse a ogni riavvio | escludere `keyring` su Android, con una prova che lo controlla |
| Verifica degli sviluppatori di Google | certa, nel 2027 | APK non installabile senza passi in più | registrarsi prima che entri in vigore in Europa |
| Keystore perso | bassa | gli utenti reinstallano e perdono le ricette | segreto del repository e copia fuori da GitHub |
| Processo chiuso durante un'operazione | media | esito da controllare a mano | già gestito (salvataggio prima della Conferma, esito incerto); in più servizio e wake lock |
| Stessa ricetta su telefono e computer (o bot) | media | date bloccate a vicenda | dirlo nell'app, come già per app e bot |
| Il portale cambia o accende l'anti-bot | come oggi | controlli fermi | come oggi: impronte, segnalazioni, aggiornamenti |

## Decisioni da prendere

1. Strada A con prototipo prima (consigliata), o direttamente B?
2. A chi va l'APK: poche persone (basta l'account gratuito a distribuzione limitata) o chiunque (registrazione
   completa)?
3. Solo arm64 o anche armv7 per i telefoni vecchi?
4. Il Play Store è un obiettivo futuro? Cambia qualche scelta (tipo di servizio, permessi, policy sui dati sanitari).

## Fonti

- Android, servizi in primo piano: <https://developer.android.com/guide/components/foreground-services>
- Google, domande sulla verifica degli sviluppatori: <https://developer.android.com/developer-verification/guides/faq?hl=en>
- Google, guida alla verifica degli sviluppatori: <https://support.google.com/android-developer-console/answer/16561738?hl=en>
- Android Authority, date dei cambiamenti all'installazione di APK: <https://www.androidauthority.com/android-sideloading-changes-timeline-3679204/>
- Android Authority, ADB e flusso avanzato: <https://www.androidauthority.com/how-to-bypass-android-developer-verification-sideloading-rules-3718002/>
- Help Net Security, paesi e date dell'obbligo: <https://www.helpnetsecurity.com/2026/06/19/android-developer-verification-rollout-markets/>
- `tauri-plugin-background-service`: <https://docs.rs/crate/tauri-plugin-background-service/0.5.3>
- Supporto dei plugin Tauri per piattaforma: `[package.metadata.platforms]` nel `Cargo.toml` di ogni plugin
  (per l'updater: Android "none").
