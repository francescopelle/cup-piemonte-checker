# App Android: piano di sviluppo

Novembre 2026. Segue l'[analisi di fattibilità](FATTIBILITA.md) e le scelte fatte dopo. Serve a chi implementa:
ogni scelta è già presa qui. Dove il piano non basta o contraddice quello che si trova, **ci si ferma e si chiede**,
non si decide da soli (vedi [Regole](#1-regole-per-chi-implementa)).

Indice: [1 Regole](#1-regole-per-chi-implementa) · [2 Decisioni](#2-decisioni) ·
[3 Cosa deve fare l'utente](#3-cosa-deve-fare-lutente) · [4 Architettura](#4-architettura) ·
[5 Versioni](#5-versioni-fissate) · [6 Crate Rust](#6-la-crate-rust-androidrust) ·
[7 Progetto Gradle](#7-il-progetto-gradle) · [8 Modulo shared](#8-modulo-shared) ·
[9 Modulo androidApp](#9-modulo-androidapp) · [10 Sicurezza](#10-sicurezza) ·
[11 Build e rilascio](#11-build-verifiche-e-rilascio) · [12 Fasi](#12-fasi) ·
[13 Collaudo](#13-collaudo-sul-telefono) · [14 Zapstore](#14-zapstore) ·
[15 Manutenzione](#15-manutenzione) · [16 Fuori dal piano](#16-fuori-dal-piano) · [Appendici](#appendice-a-file-verificati)

## 1. Regole per chi implementa

1. **Una fase, una pull request**, nell'ordine del capitolo 12. Una fase è finita quando tutti i suoi criteri di
   accettazione sono veri e la CI è verde. Non si comincia la fase dopo prima.
2. **Si tocca solo**: `android/**`, `.github/workflows/android-*.yml`, `.github/dependabot.yml`, e (solo nella
   fase 7) una sezione in fondo a `README.md`. **Mai** `app/**`, `telegram-bot/**`, `docs/**`, gli altri workflow,
   gli script in `.github/scripts/`. Lo controlla `android/scripts/verifica-repo.sh` nella CI.
3. **Niente scelte nuove.** Librerie, versioni, nomi, permessi, testi e comportamenti sono quelli scritti qui. Se una
   versione non esiste più o non si risolve, vale la regola di ripiego scritta accanto; se non c'è, ci si ferma.
4. **Mai indebolire una verifica per farla passare**: non si tolgono test, non si allargano elenchi di permessi o di
   file ammessi, non si mette `continue-on-error`, non si abbassano i livelli di lint. Se una verifica fallisce, o
   si corregge il codice o ci si ferma e si spiega.
5. **Mai dati personali fuori dall'archivio cifrato.** Codice fiscale, numero ricetta, nome della ricetta,
   prestazione, date e luoghi della visita, email e cellulare non vanno in: log (`Log.*`, `println`), registro
   dell'app, nomi di file, `SharedPreferences`, messaggi di commit, issue, output della CI. Nel registro le ricette
   si chiamano con il loro numero ("ricetta 2"), come fa già il Rust.
6. **Mai segreti nel repository o nei log**: keystore, password, `nsec`, file `.env`. Chi implementa non li vede
   mai: li mette l'utente nei segreti di GitHub (capitolo 3).
7. **Codice in italiano**, come il resto del repository: nomi di tipi, funzioni, file e commenti. Messaggi di commit
   in italiano, forma "App Android: …".
8. **Il Rust è la fonte del comportamento.** Regole su controlli, spostamenti, prenotazioni, disdette, esiti incerti
   e 20 minuti stanno in `app/core` e `app/desktop/src`: in Kotlin non si riscrivono e non si "migliorano".
9. Gli spezzoni Rust di questo piano sono stati compilati e provati. Quelli Kotlin, Gradle e YAML **no** (qui non
   si poteva scaricare l'SDK di Android): se non compilano si corregge la sintassi **senza cambiare il
   comportamento descritto**.
10. Alla fine di ogni fase si riporta: cosa è stato fatto, l'esito di ogni criterio di accettazione con il comando
    usato, e cosa resta aperto.

## 2. Decisioni

| # | Decisione | Perché |
|---|---|---|
| D1 | App **Kotlin Multiplatform** (interfaccia in Compose Multiplatform), un solo target per ora: Android. | Scelta dell'utente. |
| D2 | Il **motore resta in Rust** e si chiama da Kotlin con **UniFFI**. Non si riscrive in Kotlin. | Il motore prenota e disdice visite vere e non si può ricollaudare senza ricette vere; così si tengono le sue 44 prove e il comportamento già visto sul portale. |
| D3 | **Nessun file di `app/` cambia.** Una crate nuova, `android/rust` (workspace a sé), include i moduli di `app/desktop/src` con `#[path]` e dipende da `app/core`. | Il repository è un fork: così le correzioni di chi mantiene l'app desktop (il portale cambia) si uniscono senza conflitti. Provato: 45 prove passano, clippy pulito. |
| D4 | Tutto in una cartella nuova **`android/`**, accanto ad `app/` e `telegram-bot/`. | Come gli altri progetti del repository: separati. |
| D5 | Background: **servizio in primo piano** (`specialUse`) con notifica fissa, **sveglia esatta** per ogni controllo, wake lock solo mentre si lavora, richiesta di esclusione dall'ottimizzazione della batteria. Niente WorkManager. | Scelta dell'utente (notifica persistente); la sveglia serve perché a schermo spento i timer si fermano. |
| D6 | **Telefoni a 64 e a 32 bit**: un solo APK con `arm64-v8a` e `armeabi-v7a`. | Lo sforzo è minimo (provato: la crate passa `cargo check` per entrambi). Un solo file è più semplice da distribuire e Zapstore lo accetta. |
| D7 | Android minimo **8.0** (`minSdk 26`), `targetSdk 35`, `compileSdk 36`. | Canali di notifica, `java.time` e avvio dei servizi in primo piano senza casi speciali; copre quasi tutti i telefoni in uso. |
| D8 | Chiave delle ricette nel **Keystore di Android** da subito. La chiave arriva al Rust attraverso un "portachiavi" nostro registrato nella libreria `keyring`. | Con Kotlin nativo costa poco; `keyring` da sola su Android userebbe un portachiavi finto e perderebbe le ricette. Provato. |
| D9 | **Tutta la rete passa dal Rust** (reqwest + rustls). In Kotlin nessuna libreria di rete, nessuna WebView. | Una sola pila TLS da controllare; i certificati aggiunti dall'utente al telefono non valgono, quindi niente intercettazioni con un proxy. |
| D10 | Distribuzione: **APK allegato a una Release di GitHub** (tag `android-vX.Y.Z`). Poi **Zapstore** (fase 10). Niente Play Store. | Scelta dell'utente. |
| D11 | Aggiornamenti: l'app **avvisa** che c'è una versione nuova e apre la pagina della Release. Non scarica e non installa APK. | Meno permessi (`REQUEST_INSTALL_PACKAGES` non serve) e meno superficie d'attacco. Chi installa da Zapstore si aggiorna da lì. |
| D12 | Le **segnalazioni dei problemi** allo sviluppatore non ci sono nella prima versione. | Arriverebbero a chi mantiene l'app desktop, che non mantiene questa. |
| D13 | Nome del pacchetto **`io.github.francescopelle.cup`**; nome mostrato **"CUP Piemonte"**. Pacchetto Kotlin del codice: `cup.piemonte`. | Il nome del pacchetto non si può più cambiare dopo la prima versione: **l'utente lo conferma prima della fase 7** (capitolo 3). |
| D14 | Versioni dell'app Android indipendenti da quella desktop, da `0.1.0`. `versionCode = X·10000 + Y·100 + Z`. Unica fonte: `android/versione.properties`. | |
| D15 | Il Rust si compila sempre in `--release`, anche per l'APK di debug. | Un modo solo, meno casi. |
| D16 | Niente librerie di navigazione, di dependency injection, di rete, di serializzazione in Kotlin. | Meno dipendenze da verificare; l'app ha quattro schermate. |
| D17 | Solo italiano. Testi in un oggetto Kotlin (`Testi.kt`), non in risorse. | |
| D18 | I segreti di firma stanno in un **ambiente GitHub con approvazione** (`rilascio-android`): ogni job che firma aspetta il "sì" dell'utente. | Chi implementa ha accesso in scrittura al repository: senza approvazione un workflow potrebbe leggere i segreti. |

## 3. Cosa deve fare l'utente

Azioni che chi implementa non può o non deve fare. Ognuna dice entro quale fase serve.

| # | Azione | Entro |
|---|---|---|
| U1 | **Accendere GitHub Actions nel fork** (scheda *Actions* → *I understand my workflows, go ahead and enable them*). La PR dell'analisi non ha fatto partire nessun controllo, quindi probabilmente sono spente. | fase 2 |
| U2 | **Consentire `dl.google.com` nell'ambiente cloud** delle sessioni (SDK e NDK di Android, librerie AndroidX). Senza, in sessione si prova solo il Rust e l'app Android si compila solo nella CI: funziona, ma ogni correzione costa un giro di CI. | fase 2 |
| U3 | **Confermare D13** (nome del pacchetto e nome mostrato). | fase 7 |
| U4 | **Creare il keystore di firma** sul proprio computer e conservarne una copia fuori da GitHub (se si perde, gli utenti devono disinstallare l'app, e perdono le ricette, per aggiornare): [comandi](#u4-keystore). | fase 7 |
| U5 | **Creare l'ambiente `rilascio-android`** (Settings → Environments), con sé stessi come *Required reviewer*, e metterci i segreti `ANDROID_KEYSTORE_BASE64` e `ANDROID_KEYSTORE_PASSWORD`. | fase 7 |
| U6 | Dare a chi implementa **l'impronta SHA-256 del certificato** (è pubblica): va in `android/firma/impronta-sha256.txt`. | fase 7 |
| U7 | In Settings → *Code security*: accendere **Dependabot alerts**, **Secret scanning** e **Push protection**. | fase 8 |
| U8 | **Collaudare** sul proprio telefono con una ricetta vera ([capitolo 13](#13-collaudo-sul-telefono)) e **pubblicare** la Release in bozza. | fase 9 |
| U9 | Per Zapstore: una chiave Nostr, la prova di proprietà del certificato fatta a mano con `zsp`, l'ambiente `zapstore` con il segreto `ZAPSTORE_SIGN_WITH` ([capitolo 14](#14-zapstore)). | fase 10 |

<a id="u4-keystore"></a>**U4, comandi** (serve un JDK; la password la sceglie l'utente e non la scrive in chat):

```bash
keytool -genkeypair -v -storetype PKCS12 -keystore cup-android.p12 -alias cup -keyalg RSA -keysize 4096 -validity 10000 -dname "CN=CUP Piemonte Android"
```

```bash
keytool -list -v -keystore cup-android.p12 -alias cup | grep 'SHA256:'
```

```bash
base64 -w0 cup-android.p12
```

Il primo crea il keystore, il secondo stampa l'impronta (U6), il terzo stampa il testo da incollare nel segreto
`ANDROID_KEYSTORE_BASE64` (su macOS: `base64 -i cup-android.p12`). Con PKCS12 la password del keystore e quella
della chiave sono la stessa: basta un segreto, `ANDROID_KEYSTORE_PASSWORD`.

## 4. Architettura

```
┌──────────────────────────── processo dell'app ────────────────────────────┐
│ androidApp (Kotlin)                                                        │
│   MainActivity ── interfaccia Compose (da shared)                          │
│   ServizioControlli (primo piano) ── ciclo: wake lock → giro() → sveglia   │
│   RicevitoreSveglia · RicevitoreAvvio · Notifiche                          │
│ shared (Kotlin Multiplatform)                                              │
│   commonMain: modello · interfaccia Motore · StatoApp · schermate · Testi  │
│   androidMain: MotoreRust (UniFFI) · ChiaveArchivio (Keystore) · Appunti   │
│ ─────────────────────────── JNA / UniFFI ───────────────────────────────── │
│ libcup_mobile.so (Rust, android/rust)                                      │
│   comandi.rs (da app/desktop/src/main.rs) · portachiavi.rs · versione.rs   │
│   #[path] → app/desktop/src/{archivio,controlli,operazioni,registro,       │
│              segnalazioni}.rs          dipendenza → app/core (cup-core)    │
└────────────────────────────────────────────────────────────────────────────┘
        │ HTTPS (reqwest + rustls)                      │ HTTPS
        ▼                                               ▼
  cup.isan.csi.it (portale)              github.com (solo latest.json delle versioni)
```

**Cartelle** (tutto nuovo):

```
android/
  PIANO.md  FATTIBILITA.md  README.md (fase 7)  SICUREZZA.md (fase 8)
  versione.properties            versione=0.1.0 · repo=francescopelle/cup-piemonte-checker
  zapstore.yaml (fase 10)
  .gitignore  .gitattributes
  settings.gradle.kts  build.gradle.kts  gradle.properties  gradlew  gradlew.bat
  gradle/libs.versions.toml  gradle/wrapper/…
  firma/impronta-sha256.txt      (fase 7; pubblica)
  rust/                          crate cup-mobile, workspace a sé, con il suo Cargo.lock
    Cargo.toml  src/{lib,tipi,comandi,portachiavi,aggiornamenti,versione}.rs  src/bin/uniffi-bindgen.rs
  shared/                        modulo KMP
    build.gradle.kts
    src/commonMain/kotlin/cup/piemonte/{modello,motore,stato,ui}/…
    src/commonTest/kotlin/…
    src/androidMain/kotlin/cup/piemonte/android/…      (+ jniLibs/ generata, non nel repository)
  androidApp/                    modulo applicazione
    build.gradle.kts  proguard-rules.pro
    src/main/{AndroidManifest.xml,kotlin/cup/piemonte/app/…,res/…}
    src/androidTest/kotlin/…
  scripts/  costruisci-rust.sh · prepara-ambiente.sh · verifica-repo.sh · verifica-apk.sh ·
            prova-avvio.sh · note-release.sh · canale.py · controllo-android.sh
.github/workflows/  android-prove.yml · android-apk.yml · android-canale.yml · android-zapstore.yml
```

**Chi fa cosa**

- **Rust**: tutto quello che parla con il portale e decide (controlli, date buone, operazioni, esiti incerti,
  archivio cifrato, registro, intervallo dei 20 minuti). Espone funzioni **bloccanti**.
- **Kotlin, `shared`**: schermate e stato dell'interfaccia; trasforma i tipi UniFFI nel modello; custodisce la chiave.
- **Kotlin, `androidApp`**: ciclo di vita Android: servizio, sveglia, accensione, notifiche, permessi.
- **Un solo `Motore` per processo**, creato in `App` (la classe `Application`) e usato da Activity e servizio. Il
  Rust rifiuta una seconda apertura della stessa cartella.

**Il ciclo dei controlli.** Sul computer un thread Rust dorme 5 secondi e riprova. Su Android il ciclo lo guida
il servizio: prende il wake lock, chiama `motore.giro()` finché ritorna `true`, rilascia, chiede
`motore.prossimoControllo()`, mette una sveglia per quell'ora e aspetta. La sveglia (o un cambiamento
dall'interfaccia) lo risveglia. `giro()` fa in Rust tutto quello che fa il corpo del ciclo di `controlli::avvia`:
controllo, salvataggio, notifiche, prenotazione automatica.

## 5. Versioni fissate

| Cosa | Versione | Se non si risolve |
|---|---|---|
| Rust | `stable` (CI: quello dei runner, `rustup update stable`) | — |
| uniffi | `=0.32.2` (provata qui) | fermarsi |
| keyring | `3` (provata 3.6.3); **non** la 4 | fermarsi |
| cargo-ndk | `3.5.4` | l'ultima 3.x |
| NDK | `28.2.13676358` (r28c) | l'ultima `28.x` di `sdkmanager --list`; mai un'altra major |
| Gradle | `8.14.3` (wrapper, con `distributionSha256Sum`) | fermarsi |
| Android Gradle Plugin | `8.11.1` | l'ultima `8.11.x` |
| Kotlin | `2.2.20` | l'ultima `2.2.x` |
| Compose Multiplatform | `1.9.0` | l'ultima `1.9.x` |
| JDK | 17 (Temurin) | — |
| `androidx.activity:activity-compose` | `1.10.1` | l'ultima `1.10.x` |
| `androidx.core:core-ktx` | `1.16.0` | l'ultima `1.16.x` |
| `org.jetbrains.androidx.lifecycle:lifecycle-viewmodel-compose` | `2.9.4` | l'ultima `2.9.x` |
| `org.jetbrains.kotlinx:kotlinx-coroutines-core` / `-android` / `-test` | `1.10.2` | l'ultima `1.10.x` |
| `net.java.dev.jna:jna` (`@aar`) | `5.17.0` | l'ultima `5.x` ≥ 5.17 (serve l'allineamento a 16 KB) |
| Test Android: `androidx.test:runner` `1.6.2`, `androidx.test.ext:junit` `1.2.1`, `androidx.test:rules` `1.6.1` | | l'ultima patch |
| build-tools / platform | `36.0.0` / `android-36` | l'ultima `36.x` |

Dipendenze Kotlin ammesse: **solo** quelle della tabella, più gli artefatti di Compose Multiplatform
(`runtime`, `foundation`, `material3`, `ui`) e `kotlin("test")`. Ogni altra è una decisione nuova (regola 3).

## 6. La crate Rust `android/rust`

### 6.1 `Cargo.toml` e moduli

`Cargo.toml`, `src/portachiavi.rs`, `src/bin/uniffi-bindgen.rs` e l'inizio di `src/lib.rs` sono
nell'[Appendice A](#appendice-a-file-verificati), già provati: si copiano così.

| File | Contenuto |
|---|---|
| `lib.rs` | `#![allow(dead_code)]`, i cinque `#[path]`, `mod aggiornamenti; mod comandi; mod portachiavi; mod tipi; mod versione;`, `uniffi::setup_scaffolding!();` |
| `aggiornamenti.rs` | Solo `pub fn senza_percorsi`, copiata **identica** da `app/desktop/src/aggiornamenti.rs` (serve a `segnalazioni.rs`). Il modulo si chiama così perché `segnalazioni.rs` la importa da `crate::aggiornamenti`. |
| `tipi.rs` | I tipi che passano a Kotlin (6.2) e le conversioni dai tipi di `archivio.rs`/`controlli.rs`. |
| `comandi.rs` | `struct Motore` e i suoi metodi (6.3): il porting dei comandi di `main.rs`. |
| `portachiavi.rs` | La chiave data da Kotlin, come "portachiavi" di `keyring`. |
| `versione.rs` | La lettura del canale delle versioni (6.4). |

### 6.2 Tipi esportati (`tipi.rs`)

```rust
#[derive(Debug, uniffi::Error)]
pub enum Sbaglio { Messaggio { testo: String } }      // + impl Display che scrive `testo`
impl From<String> for Sbaglio { … }                    // così i `?` su Result<_, String> funzionano

#[derive(uniffi::Enum)]   pub enum AttivitaM { Controllo, Spostamento, Prenotazione, Disdetta }
#[derive(uniffi::Enum)]   pub enum Cosa { Ricetta, CodiceFiscale }
#[derive(uniffi::Record)] pub struct VisitaM { pub quando: String, pub luogo: String, pub cosa: String,
                                               pub disdicibile: bool }          // = !iso.is_empty()
#[derive(uniffi::Record)] pub struct DataM { pub chiave: String, pub quando: String, pub luogo: String,
                                             pub proposta: bool }
#[derive(uniffi::Record)] pub struct EsitoM { pub ts: i64, pub controllo: bool, pub attuale: Option<VisitaM>,
    pub date: u32, pub buone: Vec<DataM>, pub area_incompleta: bool, pub errore: Option<String>,
    pub nota: Option<String>, pub note_visita: Vec<String>, pub cosa: String, pub solo_proposta: bool }
#[derive(uniffi::Record)] pub struct SchedaM { pub id: u64, pub nome: String,
    pub cf: String, pub nre: String,                 // mascherati ("RSS…19X"), mai interi
    pub dove: String, pub valore: String, pub evita: Vec<u32>, pub zona: String, pub giorni: String,
    pub attiva: bool, pub nuova: bool, pub in_corso: Option<AttivitaM>, pub incerta: Option<String>,
    pub email: Option<String>, pub cellulare: Option<String>,   // mascherati
    pub automatica: bool,
    pub prossimo: Option<i64>,                       // secondi dal 1970: l'ora la formatta Kotlin
    pub esito: Option<EsitoM> }
#[derive(uniffi::Record)] pub struct ElencoM { pub ricette: Vec<SchedaM>, pub avviso: Option<String> }
#[derive(uniffi::Record)] pub struct ContattiM { pub email: String, pub cellulare: String }
#[derive(uniffi::Record)] pub struct NuovaRicetta { pub nome: String, pub cf: String, pub nre: String,
    pub dove: String, pub valore: String, pub evita: Vec<u32>, pub attiva: bool, pub email: String,
    pub cellulare: String, pub automatica: bool }
#[derive(uniffi::Record)] pub struct ModificaRicetta { pub id: u64, pub nome: String, pub dove: String,
    pub valore: String, pub evita: Vec<u32>, pub email: String, pub cellulare: String, pub automatica: bool,
    pub automatica_prima: bool }
#[derive(uniffi::Record)] pub struct Avviso { pub ricetta: u64, pub tipo: String, pub titolo: String,
                                              pub testo: String }
#[derive(uniffi::Record)] pub struct VersioneNuova { pub versione: String }

#[uniffi::export(with_foreign)]
pub trait Ascoltatore: Send + Sync {
    fn aggiornato(&self);                // i dati sono cambiati: l'interfaccia rilegge `ricette()`
    fn notifica(&self, avviso: Avviso);  // da mostrare come notifica di sistema
}
```

`SchedaM` si costruisce come `scheda()` di `main.rs` (stesse regole per `zona`, `giorni`, `mascherato`), tranne
`prossimo` ed `esito.ts`, che restano numeri. `Avviso.tipo` è `Notifica.tipo`; `Avviso.ricetta` è `Notifica.ricetta`.

### 6.3 `Motore` (`comandi.rs`)

```rust
#[derive(uniffi::Object)]
pub struct Motore { stato: Arc<controlli::Stato>, ascoltatore: Arc<dyn Ascoltatore>,
                    interrotte: Mutex<Vec<Avviso>>, cartella: PathBuf, versione: String }
```

Tutti i metodi sono `#[uniffi::export]`, **sincroni e bloccanti** (i thread li dà Kotlin).

| Metodo | Da `main.rs` | Note |
|---|---|---|
| `#[uniffi::constructor] apri(cartella_dati, cartella_registro, chiave: String, versione: String, ascoltatore) -> Result<Arc<Motore>, Sbaglio>` | `prepara` + inizio di `setup` | Vedi sotto. |
| `interrotte() -> Vec<Avviso>` | `interrotte` di `setup` | Le restituisce una volta sola (svuota). |
| `messo_da_parte() -> Option<String>` | `impostazioni` | Solo il nome del file, non il percorso. |
| `ricette() -> ElencoM` | `ricette` | |
| `contatti_di(id: Option<u64>, altra: bool) -> Option<ContattiM>` | `contatti_di` | |
| `valore_intero(id: u64, cosa: Cosa) -> Result<String, Sbaglio>` | `copia` | Ritorna il valore; negli appunti lo mette Kotlin. Errore "Ricetta non trovata." |
| `aggiungi(r: NuovaRicetta) -> Result<String, Sbaglio>` | `aggiungi` + `leggi_ricetta` | Va in rete. |
| `modifica(m: ModificaRicetta) -> Result<(), Sbaglio>` | `modifica` | |
| `elimina(id)`, `attiva(id, si: bool)`, `chiudi_incerta(id)` `-> Result<(), Sbaglio>` | omonimi | |
| `sposta(id, chiave: String)`, `prenota(id, chiave: String)`, `disdici(id)` `-> Result<String, Sbaglio>` | omonimi + `finita`, `sposta_sul_portale`, `prenota_sul_portale` | `Ok(messaggio)` se `fine.ok`, altrimenti `Err(messaggio)`; la notifica di `fine.notifica` va all'ascoltatore. |
| `giro() -> bool` | corpo del ciclo di `controlli::avvia` + `da_sola` | Vedi sotto. |
| `prossimo_controllo() -> Option<i64>` | — | Il minimo di `controlli::prossimo(r, adesso())` sulle ricette; `None` se nessuna è attiva. |
| `attive() -> u32` | — | Quante ricette hanno `attiva`. |
| `occupata() -> bool` | `esci_se_libera` | Vero se in `in_corso` c'è un'attività con `operazione()`. |
| `registro() -> Vec<String>` | `registro` | Le ultime 40 righe, dalla più recente. |
| `annota(testo: String)` | — | Scrive una riga nel registro (eventi di Android: vedi 9.7). |
| `controlla_versione(indirizzo: String) -> Result<Option<VersioneNuova>, Sbaglio>` | — | 6.4. |

Funzioni libere: `versione_motore() -> String` (`env!("CARGO_PKG_VERSION")`) e `ora_locale() -> String`
(`chrono::Local::now().format("%Y-%m-%d %H:%M")`), che serve a una prova sull'emulatore.

**Regole di porting** (valgono per ogni metodo):

- `app.emit("ricette", ())` → `self.ascoltatore.aggiornato()`.
- `avvisa(app, titolo, testo, per_registro)` → `self.avvisa(&notifica)`: chiama `ascoltatore.notifica(…)` e scrive
  nel registro `notifica: ricetta {id}, {tipo}`. Mai titolo e testo nel registro.
- `tauri::async_runtime::spawn_blocking(…).await` → chiamata diretta.
- `stato: Condiviso` → `self.stato`. `salva(&app, &stato, &d)` → `self.stato.salva(&d)` e poi `aggiornato()`.
- Testi dei messaggi e delle righe del registro: **identici** a quelli di `main.rs`.
- Le funzioni private di `main.rs` che servono (`mascherato`, `nome_pulito`, `contatti_puliti`, `leggi_ricetta`,
  `sposta_sul_portale`, `prenota_sul_portale`) e `altre_buone` di `controlli.rs` (privata) si copiano identiche.
- Non si porta: `copia` negli appunti, `impostazioni`/`imposta_avvio`, tutto ciò che riguarda `aggiornamenti` e
  `segnalazioni` (non si chiama mai `segnalazioni::avvia`: le chiamate a `segnalazioni::segnala` dentro i moduli
  inclusi restano e non fanno nulla), l'icona, la finestra.

**`apri`**, nell'ordine:

1. Se `cartella_dati` è già aperta in questo processo (un `static APERTE: Mutex<HashSet<PathBuf>>`), errore
   "L'archivio è già aperto." `Drop` di `Motore` toglie la cartella dall'insieme.
2. `chiave` deve essere 64 caratteri esadecimali minuscoli, altrimenti errore "Chiave non valida."
3. Se in `cartella_dati` esiste `chiave.segreta`: errore "Trovata una chiave in un file: su Android non è
   prevista." (Mai una chiave in chiaro sul telefono.)
4. `portachiavi::installa(chiave)`, poi `Archivio::apri`, poi si controlla `protezione == Protezione::Portachiavi`.
5. Come `prepara`: `leggi`, `operazioni::riprendi`, righe del registro, `scrivi`. Le notifiche di `riprendi`
   diventano `interrotte`.
6. Registro: `avvio, versione {versione} (android)` all'inizio e `avvio completato` alla fine.

**`giro`** (ritorna `true` se ha controllato una ricetta):

```rust
pub fn giro(&self) -> bool {
    let a = self.ascoltatore.clone();
    match catch_unwind(AssertUnwindSafe(|| controlli::giro(&self.stato, &controlli::sul_portale, &|| a.aggiornato()))) {
        Ok(Some(g)) => {
            for n in &g.notifiche { self.avvisa(n); }
            self.ascoltatore.aggiornato();
            if let Some((id, chiave)) = g.da_sola {
                let riuscita = catch_unwind(AssertUnwindSafe(|| self.da_sola(id, &chiave))).unwrap_or_else(|_| {
                    self.stato.registro.scrivi("errore interno nella prenotazione automatica");
                    false
                });
                if !riuscita && altre_buone(&self.stato, id, &chiave) {
                    for n in &g.rinviate { self.avvisa(n); }
                }
                self.ascoltatore.aggiornato();
            }
            true
        }
        Ok(None) => false,
        Err(_) => {
            controlli::blocca(&self.stato.in_corso).retain(|_, a| *a != controlli::Attivita::Controllo);
            self.stato.registro.scrivi("errore interno nel giro dei controlli");
            self.ascoltatore.aggiornato();
            false
        }
    }
}
```

`self.da_sola(id, chiave)` è `da_sola` di `main.rs` con le regole di porting.

### 6.4 `versione.rs`

`controlla_versione(indirizzo)`:

1. `indirizzo` deve cominciare con `https://github.com/`, altrimenti errore.
2. `reqwest::blocking::Client::builder().https_only(true).timeout(20 s).build()`; `GET`; stato 200, altrimenti
   errore "Non riesco a leggere le versioni."; si leggono al massimo 4096 byte.
3. JSON `{"versione": "0.1.1", "codice": 101}` (`serde`, campi in più ignorati). `versione` deve essere tre numeri
   separati da punti, ognuno ≤ 99, e `codice` deve valere `X·10000 + Y·100 + Z`: altrimenti errore.
4. Ritorna `Some(VersioneNuova)` solo se è **maggiore** di `self.versione` (confronto numerico dei tre numeri).

Dal canale si prende **solo il numero di versione**: mai indirizzi, mai testo da mostrare.

### 6.5 Prove Rust

In `android/rust/src`, oltre alle 44 che arrivano dai moduli inclusi:

- `la_chiave_data_da_fuori_apre_e_riapre` (Appendice A): salva, riapre con la stessa chiave, e con un'altra chiave
  l'archivio viene messo da parte.
- `seconda_apertura_rifiutata`, `chiave_non_valida`, `chiave_in_un_file_rifiutata`.
- `scheda_mascherata`: `cf`, `nre`, `email` e `cellulare` di `SchedaM` non contengono mai i valori interi.
- `senza_percorsi_come_desktop`: stessi casi della prova di `aggiornamenti.rs` dell'app desktop.
- `giro_senza_ricette_dovute` ritorna `false`; `prossimo_controllo` e `attive` con ricette attive e in pausa.
- `versione`: `piu_nuova` (0.1.1 > 0.1.0, 0.2.0 > 0.1.9, uguale no, più vecchia no); JSON non valido, versione con
  lettere, `codice` che non corrisponde, indirizzo non `https://github.com/`.
- `registro_senza_dati_personali`: dopo `aggiungi` fallita e `modifica`, nel registro non compaiono codice fiscale,
  numero ricetta, nome, email, cellulare usati nella prova.

Comandi, da `android/rust`: `cargo test --locked` e `cargo clippy --all-targets --locked -- -D warnings`. Senza
NDK si controlla la compilazione per Android con `android/scripts/controllo-android.sh`
([Appendice B](#appendice-b-controllo-per-android-senza-ndk)).

## 7. Il progetto Gradle

- `settings.gradle.kts`: `rootProject.name = "cup-piemonte-android"`, moduli `:shared` e `:androidApp`,
  repository `google()`, `mavenCentral()`, `gradlePluginPortal()` in `pluginManagement`, e `google()`,
  `mavenCentral()` in `dependencyResolutionManagement` con `RepositoriesMode.FAIL_ON_PROJECT_REPOS`.
- `gradle/libs.versions.toml`: le versioni del capitolo 5.
- `gradle.properties`: `org.gradle.jvmargs=-Xmx3g`, `android.useAndroidX=true`, `kotlin.code.style=official`,
  `android.nonTransitiveRClass=true`.
- `gradle/wrapper/gradle-wrapper.properties`: Gradle 8.14.3 con `distributionSha256Sum` (il valore è nel file
  `gradle-8.14.3-bin.zip.sha256` su `services.gradle.org/distributions/`). Il wrapper si genera con
  `gradle wrapper --gradle-version 8.14.3` e `gradle-wrapper.jar` va nel repository.
- `.gitattributes`: `*.jar binary`, `gradlew.bat text eol=crlf`.
- `.gitignore`: `.gradle/`, `build/`, `local.properties`, `**/jniLibs/`, `rust/target/`, `*.jks`, `*.p12`,
  `*.keystore`, `keystore.properties`, `.env`, `*.apk`, `*.aab`.
- `versione.properties`: letto da Gradle (in `androidApp/build.gradle.kts`) per `versionName`, `versionCode` e
  `buildConfigField("String", "REPO", …)`.

**`shared/build.gradle.kts`**: plugin `kotlinMultiplatform`, `com.android.library`, `composeMultiplatform`,
`org.jetbrains.kotlin.plugin.compose`. `kotlin { androidTarget { compilerOptions { jvmTarget = JVM_17 } } }`.
`android { namespace = "cup.piemonte.shared"; compileSdk = 36; defaultConfig { minSdk = 26 }; compileOptions 17 }`.
`commonMain`: Compose (`runtime`, `foundation`, `material3`, `ui`), coroutines, lifecycle-viewmodel-compose.
`commonTest`: `kotlin("test")`, `kotlinx-coroutines-test`. `androidMain`: `net.java.dev.jna:jna:5.17.0@aar`,
`core-ktx`, `kotlinx-coroutines-android`; in più `kotlin.srcDir(layout.buildDirectory.dir("generated/uniffi"))`.

Compito Gradle `costruisciRust` (`Exec`, in `shared`): esegue `../scripts/costruisci-rust.sh`; ingressi
dichiarati: `rust/src`, `rust/Cargo.toml`, `rust/Cargo.lock`, `../app/core/src`, `../app/desktop/src`; uscite:
`src/androidMain/jniLibs` e `build/generated/uniffi`. `preBuild` e tutti i compiti `KotlinCompile` di `shared`
dipendono da lui.

**`scripts/costruisci-rust.sh`** (`set -euo pipefail`), da `android/rust`:

1. `cargo ndk --platform 26 -t arm64-v8a -t armeabi-v7a -t x86_64 -o ../shared/src/androidMain/jniLibs build --release --locked --lib`
2. `cargo build --release --locked --lib` (per l'host: serve al generatore).
3. `cargo run --release --locked --features bindgen --bin uniffi-bindgen -- generate --library target/release/libcup_mobile.so --language kotlin --no-format --out-dir ../shared/build/generated/uniffi`

In `android/rust/.cargo/config.toml`, per l'allineamento a 16 KB (Android 15 e successivi):

```toml
[target.aarch64-linux-android]
rustflags = ["-C", "link-arg=-Wl,-z,max-page-size=16384"]
[target.x86_64-linux-android]
rustflags = ["-C", "link-arg=-Wl,-z,max-page-size=16384"]
```

**`androidApp/build.gradle.kts`**: plugin `com.android.application`, `org.jetbrains.kotlin.android`,
`org.jetbrains.kotlin.plugin.compose`. `namespace = "cup.piemonte.app"`,
`applicationId = "io.github.francescopelle.cup"`, `minSdk 26`, `targetSdk 35`, `compileSdk 36`,
`ndkVersion` del capitolo 5, `buildFeatures { compose = true; buildConfig = true }`,
`lint { abortOnError = true; checkReleaseBuilds = true }` (elenco dei controlli fatali in 10.3).

| Tipo di build | Id | Nome mostrato | ABI | Firma | Altro |
|---|---|---|---|---|---|
| `debug` | `….cup.debug` | "CUP (debug)" | arm64-v8a, armeabi-v7a, x86_64 | chiave di debug | `debuggable`, schermate catturabili |
| `release` | `….cup` | "CUP Piemonte" | arm64-v8a, armeabi-v7a | keystore dai segreti | `isMinifyEnabled = true`, `isShrinkResources = true` |
| `collaudo` | `….cup.prova` | "CUP Piemonte (prova)" | come `debug` | keystore dai segreti se ci sono, altrimenti debug | `initWith(release)`, `versionNameSuffix = "-prova"`, `matchingFallbacks += "release"` |

Le ABI si filtrano con `ndk { abiFilters += … }` dentro ogni tipo di build. La firma `release` legge
`ANDROID_KEYSTORE_FILE` e `ANDROID_KEYSTORE_PASSWORD` dall'ambiente (alias `cup`, password della chiave uguale);
se mancano, `assembleRelease` fallisce con un messaggio chiaro.

`proguard-rules.pro`:

```
-dontwarn java.awt.**
-keep class com.sun.jna.** { *; }
-keepclassmembers class * extends com.sun.jna.** { public *; }
-keep class uniffi.** { *; }
```

## 8. Modulo `shared`

### 8.1 Modello e interfacce (`commonMain`)

- `modello/`: `Scheda`, `Esito`, `Visita`, `DataBuona`, `Attivita`, `Contatti`, `NuovaRicetta`, `ModificaRicetta`,
  `Avviso`, `Elenco`: gli stessi campi dei tipi `…M` del Rust, con `Long` per id e orari e `Int` per i giorni.
- `motore/Motore.kt`: `interface Motore` con una funzione `suspend` per ogni metodo di 6.3 tranne `giro`,
  `prossimoControllo`, `attive`, `occupata`, `annota` (che stanno in `interface MotoreDiServizio : Motore`, non
  `suspend`), più `val cambi: SharedFlow<Unit>` (un colpo per ogni `aggiornato()`). Gli errori sono
  `class ErroreMotore(val testo: String) : Exception(testo)`.
- `motore/Piattaforma.kt`: `interface Piattaforma` con `fun copiaSensibile(testo: String)`,
  `fun apriIndirizzo(indirizzo: String)`, `fun ora(ts: Long): String` ("14:05" oggi, "21/10 14:05" un altro
  giorno), `fun statoPermessi(): StatoPermessi` (`notifiche`, `batteriaEsclusa`, `sveglieEsatte`: tre `Boolean`),
  `fun chiedi(cosa: Permesso)`, `val versione: String`, `var controlliFermi: Boolean`,
  `var benvenutoVisto: Boolean`, `fun svegliaServizio()`.

### 8.2 `androidMain`

- **`MotoreRust`**: implementa `MotoreDiServizio` sopra `uniffi.cup_mobile.Motore`. Le funzioni `suspend` girano
  in `withContext(Dispatchers.IO)`; `sposta`, `prenota`, `disdici`, `aggiungi` in più dentro
  `NonCancellable` e con un wake lock parziale `cup:operazione` (scadenza 10 minuti, rilasciato in `finally`).
  `Sbaglio` → `ErroreMotore(testo)`; ogni altra eccezione di UniFFI (`InternalException`) →
  `ErroreMotore("Errore interno dell'app.")` e una riga nel registro `errore interno: {nome della classe}`.
  L'`Ascoltatore` inoltra `aggiornato()` a `cambi` e `notifica()` a `Notificatore` (un'interfaccia; la
  implementa `androidApp`).
- **`ChiaveArchivio`** (10.2, requisito S4).
- **`PiattaformaAndroid`**: appunti (S9), `Intent.ACTION_VIEW`, `java.time`, permessi, `SharedPreferences`
  `cup_preferenze` (solo `controlli_fermi`, `benvenuto_visto`, `versione_avvisata`, `ultimo_controllo_versione`).

### 8.3 Stato dell'interfaccia (`commonMain`, `stato/StatoApp.kt`)

`class StatoApp(motore: Motore, piattaforma: Piattaforma) : ViewModel()` con uno `StateFlow<Vista>`:

```kotlin
data class Vista(
    val schermata: Schermata,            // Benvenuto, Principale, Modulo(idInModifica: Long?), Impostazioni
    val ricette: List<Scheda>, val avviso: String?,           // avviso = salvataggio non riuscito
    val messaggio: Messaggio?,           // testo + errore sì/no, come `avvisa()` di app.js
    val daConfermare: Conferma?,         // Sposta(id, data) | Prenota(id, data) | Disdici(id)
    val daEliminare: Long?,              // "Sicuro? Elimina" per 5 secondi
    val inViaggio: Set<Long>,            // operazioni partite da qui, contro il doppio tocco
    val permessi: StatoPermessi, val controlliFermi: Boolean, val versioneNuova: String?,
    val registro: List<String>, val salvataggioInCorso: Boolean,
)
```

Rilegge `ricette()` a ogni evento di `cambi`, a ogni ritorno in primo piano e ogni 60 secondi mentre è visibile.
Le regole sono quelle di `app/desktop/ui/app.js`, funzione per funzione: `riepilogo`, `operazione`,
`riquadroConferma` (compresa la regola `valida`), `scheda`, `sceltaDove`, il `submit` del modulo, i due tocchi di
"Elimina". Le parti pure (testo del riepilogo, `sceltaDove`, `valida`, costruzione di `dove`/`valore` dal modulo)
stanno in funzioni senza stato, in `stato/Regole.kt`, con le loro prove in `commonTest`.

### 8.4 Schermate (`commonMain`, `ui/`)

Material 3, colore primario `#0B6BCB`, tema chiaro o scuro come il sistema, niente colori dinamici. Una colonna
che scorre, larga al massimo 560 dp, margini 16 dp, `WindowInsets.safeDrawing`. Pulsanti alti almeno 48 dp.

**Testi**: quelli di `app/desktop/ui/index.html` e `app.js`, **copiati alla lettera**, con solo queste
sostituzioni: "computer" → "telefono"; "non spegnere il computer" → "non spegnere il telefono"; "clicca"/"tocchi"
restano "tocchi". I testi nuovi di Android sono qui sotto, tra virgolette, e si usano così.

1. **Errore di avvio** (l'archivio non si apre): titolo "CUP Piemonte non riesce a partire", il messaggio, e:
   - chiave mancante o illeggibile con archivio presente: "Le ricette salvate non si possono più aprire: la chiave
     del telefono non c'è più (succede dopo un ripristino o un cambio di telefono)." e il pulsante
     "Ricomincia da capo", che chiede conferma ("Le ricette salvate verranno messe da parte e non saranno più
     leggibili. Continuare?") e poi crea una chiave nuova;
   - ogni altro errore: pulsante "Riprova".
2. **Benvenuto** (solo la prima volta): "CUP Piemonte controlla il portale del CUP ogni 20 minuti e ti avvisa
   quando esce una data prima della tua prenotazione." · "Non è un'app ufficiale: non è collegata alla Regione
   Piemonte né al CSI Piemonte." · "Codice fiscale e ricetta restano su questo telefono, cifrati, e vanno solo al
   portale del CUP." · poi tre righe con stato (✓ o da fare) e pulsante: "Consenti le notifiche" (solo Android 13+),
   "Lascia lavorare l'app a schermo spento" (esclusione dalla batteria), "Consenti le sveglie precise" (solo se
   `sveglieEsatte` è falso) · "Continua".
3. **Principale**: barra con "CUP Piemonte" e ⚙. Poi, se servono, i riquadri di avviso, in quest'ordine:
   `avviso` (salvataggio); "I controlli sono fermi." + "Riprendi"; "Le notifiche sono spente: non saprai quando
   esce una data." + "Consenti"; "Il telefono può fermare l'app a schermo spento." + "Sistema"; "C'è la versione
   X di CUP Piemonte." + "Apri la pagina". Poi `messaggio`, le schede, "Nessuna ricetta. …" (testo desktop), e il
   pulsante "+ Aggiungi una ricetta".
4. **Scheda**: come `scheda()` di `app.js`, stessi elementi nello stesso ordine, stesse emoji. "Copia" mostra
   "Copiato il … : incollalo sul sito del CUP. Tra 2 minuti l'app svuota gli appunti."
5. **Modulo** (schermata intera, Indietro = "Annulla"): gli stessi campi di `index.html`, nello stesso ordine.
   Codice fiscale e NRE: tastiera senza correzione automatica, maiuscole. Nota dei contatti: "…Restano cifrati su
   questo telefono…".
6. **Impostazioni** (schermata intera): stato di notifiche, batteria e sveglie precise, ognuno con il pulsante che
   apre la pagina di sistema giusta; "Se i controlli si fermano a schermo spento, il tuo telefono potrebbe avere
   regole in più: dontkillmyapp.com spiega cosa fare per ogni marca." + "Apri"; "Ferma i controlli"/"Riprendi i
   controlli"; "🔒 Le ricette sono salvate cifrate su questo telefono, con la chiave nel Keystore di Android.";
   "Versione X" + "Controlla ora" (messaggi: "C'è la versione X." / "È l'ultima versione."); il file messo da
   parte, se c'è; "Registro dell'app" (ultime 40 righe) + "Copia il registro".

Tasto Indietro: da Modulo e Impostazioni torna a Principale; da Principale esce (il servizio continua).

### 8.5 Prove (`commonTest`)

Con un `MotoreFinto` e una `PiattaformaFinta`: testo del riepilogo in tutti i casi di `riepilogo()`; `sceltaDove`
e il suo inverso; `valida` (la data sparisce dopo un controllo → il riquadro si chiude); doppio tocco che non fa
partire due operazioni; "Elimina" in due tocchi entro 5 secondi; `automaticaPrima`; "altro" senza comune →
"Scrivi il comune in cui cercare."; errore di `aggiungi` mostrato e modulo che resta aperto.

## 9. Modulo `androidApp`

### 9.1 Manifest

```xml
<manifest xmlns:android="http://schemas.android.com/apk/res/android"
    xmlns:tools="http://schemas.android.com/tools">
  <uses-permission android:name="android.permission.INTERNET" />
  <uses-permission android:name="android.permission.POST_NOTIFICATIONS" />
  <uses-permission android:name="android.permission.FOREGROUND_SERVICE" />
  <uses-permission android:name="android.permission.FOREGROUND_SERVICE_SPECIAL_USE" />
  <uses-permission android:name="android.permission.RECEIVE_BOOT_COMPLETED" />
  <uses-permission android:name="android.permission.WAKE_LOCK" />
  <uses-permission android:name="android.permission.REQUEST_IGNORE_BATTERY_OPTIMIZATIONS" />
  <uses-permission android:name="android.permission.SCHEDULE_EXACT_ALARM" android:maxSdkVersion="32" />
  <uses-permission android:name="android.permission.USE_EXACT_ALARM" />

  <application android:name=".App" android:label="@string/nome_app" android:icon="@mipmap/ic_launcher"
      android:allowBackup="false" android:fullBackupContent="false"
      android:dataExtractionRules="@xml/niente_backup" android:usesCleartextTraffic="false"
      android:networkSecurityConfig="@xml/rete" android:supportsRtl="false"
      android:theme="@style/Tema.Cup">
    <activity android:name=".MainActivity" android:exported="true" android:launchMode="singleTask"
        android:windowSoftInputMode="adjustResize">
      <intent-filter>
        <action android:name="android.intent.action.MAIN" />
        <category android:name="android.intent.category.LAUNCHER" />
      </intent-filter>
    </activity>
    <service android:name=".ServizioControlli" android:exported="false"
        android:foregroundServiceType="specialUse">
      <property android:name="android.app.PROPERTY_SPECIAL_USE_FGS_SUBTYPE"
          android:value="Controllo periodico delle date libere sul portale CUP Piemonte, chiesto dall'utente" />
    </service>
    <receiver android:name=".RicevitoreSveglia" android:exported="false" />
    <receiver android:name=".RicevitoreAvvio" android:exported="false">
      <intent-filter>
        <action android:name="android.intent.action.BOOT_COMPLETED" />
        <action android:name="android.intent.action.MY_PACKAGE_REPLACED" />
      </intent-filter>
    </receiver>
    <!-- arriva da una libreria AndroidX ed e' esportato: all'app non serve -->
    <receiver android:name="androidx.profileinstaller.ProfileInstallReceiver" tools:node="remove" />
  </application>
</manifest>
```

Sono **tutti** i permessi e i componenti dell'app: l'elenco lo controlla `verifica-apk.sh`. Le librerie AndroidX
ne aggiungono due, ammessi e da non togliere: il permesso interno
`<id dell'app>.DYNAMIC_RECEIVER_NOT_EXPORTED_PERMISSION` (di `androidx.core`, protetto dalla firma) e il provider
non esportato `androidx.startup.InitializationProvider`. Ogni altro permesso o componente che compare nell'APK è
un errore: si toglie con `tools:node="remove"` se viene da una libreria ammessa, altrimenti ci si ferma.
`res/xml/rete.xml`:
`<network-security-config><base-config cleartextTrafficPermitted="false" /></network-security-config>`.
`res/xml/niente_backup.xml`: `<data-extraction-rules>` con `<cloud-backup>` e `<device-transfer>` che escludono
ogni dominio (`root`, `file`, `database`, `sharedpref`, `external`).

### 9.2 `App` e `Contenitore`

`App.onCreate` crea solo i canali di notifica. `Contenitore` (un `object`) apre il motore **una volta**, al primo
`Contenitore.motore(context)` (in `synchronized`), e ricorda l'esito (`Aperto(motore)` o `Errore(tipo, testo)`):

1. cartelle: `noBackupFilesDir/dati` e `noBackupFilesDir/registro`;
2. `ChiaveArchivio.leggi(…)`; se è `Mancante` e `dati/ricette.cifrate` non esiste → `ChiaveArchivio.crea(…)`;
   se è `Mancante` o `Illeggibile` e l'archivio esiste → `Errore(Chiave, …)` (schermata 1 di 8.4);
3. `Motore.apri(dati, registro, chiave, BuildConfig.VERSION_NAME, ascoltatore)`;
4. le `interrotte()` si mostrano come notifiche; `Log.i("CUP", "avvio completato")` (l'unico `Log` dell'app: lo
   aspetta `prova-avvio.sh`).

"Ricomincia da capo" chiama `ChiaveArchivio.crea` e poi riapre: il Rust mette da parte l'archivio che non decifra.

### 9.3 `ServizioControlli`

Costanti: `ID_FISSA = 1`, azioni `cup.GIRO` e `cup.FERMA`.

- `onCreate`: `ServiceCompat.startForeground(this, ID_FISSA, notificaFissa(), tipo)` con
  `tipo = FOREGROUND_SERVICE_TYPE_SPECIAL_USE` da Android 14, `0` prima. Poi avvia il ciclo.
- `onStartCommand`: `cup.FERMA` → se `motore.occupata()` mostra la notifica "CUP Piemonte: aspetta un momento" /
  "C'è uno spostamento, una prenotazione o una disdetta in corso: riprova tra qualche minuto." e continua;
  altrimenti `controlliFermi = true`, annulla la sveglia, `stopSelf()`. Ogni altra azione → `segnale.trySend(Unit)`.
  Ritorna sempre `START_STICKY`.
- Il ciclo (una coroutine su un thread solo: `Dispatchers.IO.limitedParallelism(1)`; `segnale` è un
  `Channel<Unit>(Channel.CONFLATED)`):

```kotlin
while (isActive) {
    if (piattaforma.controlliFermi || motore.attive() == 0) { sveglia.annulla(); stopSelf(); break }
    wakeLock.acquire(10 * 60_000L)                         // "cup:controllo", non contato
    try { while (motore.giro()) { aggiornaFissa(); wakeLock.acquire(10 * 60_000L) } }
    finally { if (wakeLock.isHeld) wakeLock.release() }
    controllaVersioneSeServe()                             // 9.6
    val prossimo = motore.prossimoControllo() ?: continue  // secondi dal 1970
    val traMs = ((prossimo * 1000) - System.currentTimeMillis()).coerceAtLeast(30_000)
    sveglia.programma(System.currentTimeMillis() + traMs)
    aggiornaFissa()
    withTimeoutOrNull(traMs) { segnale.receive() }
}
```

- Se il motore non si apre: una notifica "CUP Piemonte non riesce a partire" / "Apri l'app per vedere cosa è
  successo." e `stopSelf()`.
- `onDestroy`: annulla la coroutine; **non** annulla la sveglia (se Android chiude il servizio, la sveglia lo
  riavvia).
- Notifica fissa (canale `controlli`): titolo "CUP Piemonte"; testo "Controllo N ricette ogni 20 minuti · prossimo
  alle HH:MM" ("1 ricetta" al singolare), oppure "Controllo in corso…" durante `giro()`; tocco → `MainActivity`;
  azione "Ferma i controlli" → `cup.FERMA`; `setOngoing(true)`, `setOnlyAlertOnce(true)`,
  `VISIBILITY_PUBLIC` (non contiene dati personali).

**Avvio del servizio** (`ServizioControlli.avvia(context)`, che fa
`ContextCompat.startForegroundService` dentro un `try`: se Android lo rifiuta, una riga nel registro e basta): da
`MainActivity.onStart`, da `RicevitoreAvvio`, da `RicevitoreSveglia`, da `piattaforma.svegliaServizio()` dopo
`aggiungi`, `attiva`, `modifica`, `elimina` e "Riprendi". Sempre e solo se `!controlliFermi`, il motore è
`Aperto` e `attive() > 0`.

### 9.4 Sveglia e ricevitori

- `Sveglia.programma(quandoMs)`: `PendingIntent.getBroadcast(context, 1, Intent(context,
  RicevitoreSveglia::class.java), FLAG_UPDATE_CURRENT or FLAG_IMMUTABLE)`. Se Android < 12 oppure
  `alarmManager.canScheduleExactAlarms()`: `setExactAndAllowWhileIdle(RTC_WAKEUP, quandoMs, pi)`; altrimenti
  `setAndAllowWhileIdle(RTC_WAKEUP, quandoMs, pi)`. `annulla()`: `alarmManager.cancel(pi)`.
- `RicevitoreSveglia.onReceive`: prende un wake lock parziale `cup:sveglia` con scadenza 30 secondi (copre il
  passaggio al servizio), poi `ServizioControlli.avvia(context)` con l'azione `cup.GIRO`.
- `RicevitoreAvvio.onReceive`: solo per `BOOT_COMPLETED` e `MY_PACKAGE_REPLACED` (ogni altra azione si ignora);
  `ServizioControlli.avvia(context)`.

### 9.5 Notifiche (`Notifiche.kt`, implementa `Notificatore`)

| Canale | Nome | Importanza | Per |
|---|---|---|---|
| `controlli` | "Controlli in corso" | `LOW` | la notifica fissa |
| `date` | "Date trovate e prenotazioni" | `HIGH` | `tipo` ∈ {`date nuove`, `spostata`, `prenotata`, `spostamento`, `prenotazione`, `disdetta`, `automatica non riuscita`, `esito incerto`, `operazione interrotta`, `già prenotata`} |
| `avvisi` | "Avvisi dell'app" | `DEFAULT` | `pausa`, ogni altro `tipo`, versione nuova, errori di avvio |

Ogni avviso: id da un `AtomicInteger` che parte da 100; titolo e testo del Rust; `BigTextStyle`;
`setAutoCancel(true)`; tocco → `MainActivity`; **`VISIBILITY_PRIVATE`** con una versione pubblica "CUP Piemonte" /
"C'è una novità: sblocca il telefono per leggerla." (S10). Tutti i `PendingIntent` sono `FLAG_IMMUTABLE` e con
intent espliciti. Se le notifiche sono negate l'avviso si perde (l'utente lo vede nell'app): nel registro
`notifica non mostrata: ricetta {id}, {tipo}`.

### 9.6 Versione nuova

`controllaVersioneSeServe()`: al massimo una volta ogni 24 ore (`ultimo_controllo_versione`), chiama
`motore.controllaVersione("https://github.com/${BuildConfig.REPO}/releases/download/android-aggiornamenti/latest.json")`.
Se c'è una versione nuova diversa da `versione_avvisata`: notifica (canale `avvisi`) "C'è la versione X di CUP
Piemonte" / "Tocca per aprire la pagina da cui scaricarla." e la ricorda. Il tocco e il pulsante dell'app aprono
`https://github.com/${BuildConfig.REPO}/releases/tag/android-vX`, costruito dall'app. Errori di rete: una riga nel
registro, niente all'utente (tranne con "Controlla ora").

### 9.7 `MainActivity` e registro

`MainActivity`: `enableEdgeToEdge()`; in `release` e `collaudo` `window.setFlags(FLAG_SECURE, FLAG_SECURE)` (S8);
`window.decorView.filterTouchesWhenObscured = true` (S13); `setContent { … }`; in `onStart`
`ServizioControlli.avvia`; i permessi con `registerForActivityResult`. Esclusione dalla batteria:
`Intent(Settings.ACTION_REQUEST_IGNORE_BATTERY_OPTIMIZATIONS, Uri.parse("package:$packageName"))`; sveglie
precise: `Settings.ACTION_REQUEST_SCHEDULE_EXACT_ALARM`; notifiche negate due volte:
`Settings.ACTION_APP_NOTIFICATION_SETTINGS`.

Righe che Kotlin scrive nel registro con `motore.annota` (testi fissi, mai dati personali): `servizio: avviato`,
`servizio: fermato dall'utente`, `servizio: avvio rifiutato da Android`, `sveglia: tra N minuti (esatta|non
esatta)`, `accensione: servizio riavviato`, `permessi: notifiche si|no, batteria si|no, sveglie si|no` (a ogni
avvio dell'Activity), `versione nuova: X`, `controllo versione non riuscito`.

### 9.8 Icone

`mipmap` dall'icona dell'app desktop (`app/desktop/icons/icon.png`, si legge soltanto) con uno script Python
(Pillow) lanciato una volta, con le immagini generate nel repository: classica a 48/72/96/144/192 px e adattiva
(primo piano 432 px con l'icona al 66% al centro, sfondo `#0B6BCB`), più `androidApp/src/main/ic_launcher-512.png`
(512 px, per Zapstore). Icona delle notifiche
`res/drawable/ic_notifica.xml`: vettoriale bianca 24 dp, il simbolo Material "event available".

## 10. Sicurezza

### 10.1 Cosa si protegge e da chi

**Beni**: codice fiscale e numero ricetta (bastano per vedere, spostare e disdire una prenotazione sul portale);
prestazione, data e luogo della visita (dati sanitari); email e cellulare; la possibilità di prenotare o disdire a
nome dell'utente; la chiave di firma dell'APK.

| Minaccia | Contromisura |
|---|---|
| Un'altra app sul telefono legge i dati | Cartella privata dell'app, archivio cifrato (XChaCha20-Poly1305) con chiave nel Keystore, nessun componente esportato tranne l'Activity di avvio, nessun file su memoria condivisa (S1-S5) |
| I dati escono con un backup | Backup e trasferimento tra dispositivi spenti; file in `noBackupFilesDir` (S3) |
| Qualcuno intercetta la rete | Solo HTTPS con rustls e le radici Mozilla incluse; i certificati installati sul telefono non valgono (S6) |
| I dati finiscono in log, notifiche, schermate, appunti | S7-S10 |
| Un'altra app fa partire un'operazione o copre i pulsanti | Nessun intent con dati in ingresso, `PendingIntent` immutabili, tocchi filtrati se coperti (S11-S13) |
| Un APK falso o modificato | Firma con chiave propria e impronta verificata nella CI; l'app non installa nulla da sé; attestazione di provenienza della build (S14-S16) |
| Una dipendenza o la CI compromesse | Dipendenze fissate e poche, `Cargo.lock`, `cargo audit`, Dependabot, segreti in un ambiente con approvazione, job di firma senza action di terze parti (S17-S20) |
| Telefono sbloccato in mano ad altri | **Non coperto** nella prima versione: chi apre l'app vede le ricette (mascherate) e può operare. Blocco con impronta: [fuori dal piano](#16-fuori-dal-piano). |

### 10.2 Requisiti

Ogni requisito ha la sua verifica; l'elenco con l'esito va in `android/SICUREZZA.md` (fase 8).

| # | Requisito | Verifica |
|---|---|---|
| S1 | L'unico componente esportato è `MainActivity`, senza `intent-filter` oltre a `MAIN`/`LAUNCHER`. Nessun `ContentProvider` esportato. | `verifica-apk.sh` |
| S2 | I permessi sono esattamente i 9 di 9.1, più quello interno di `androidx.core` descritto lì. | `verifica-apk.sh` |
| S3 | `allowBackup="false"`, regole che escludono tutto, dati solo in `noBackupFilesDir`. | `verifica-apk.sh`; prova strumentata: dopo aver salvato una ricetta finta, `filesDir`, `cacheDir` ed `externalCacheDir` sono vuoti |
| S4 | **Chiave dell'archivio**: 32 byte da `SecureRandom`, cifrati con una chiave AES-256-GCM del Keystore (alias `cup_archivio_v1`, non esportabile, senza autenticazione dell'utente, senza StrongBox). File `noBackupFilesDir/chiave.protetta` = 1 byte di versione (`0x01`) + 12 byte di IV + testo cifrato e tag; scritto su un file temporaneo e rinominato; **riletto e decifrato subito dopo**, e se non coincide si fallisce. Mai una chiave nuova se l'archivio esiste. L'array con la chiave si azzera dopo l'uso. | prove strumentate: crea → rileggi; file troncato → `Illeggibile`; alias cancellato → `Illeggibile`; archivio presente e chiave assente → nessuna chiave nuova |
| S5 | Mai `chiave.segreta` (chiave in chiaro) su Android; dopo `apri` la protezione è `Portachiavi`. | prove Rust (6.5) |
| S6 | Rete solo dal Rust. `usesCleartextTraffic="false"`. In Kotlin nessun uso di `java.net`, `HttpURLConnection`, `okhttp`, `ktor`, `WebView`. `controlla_versione` con `https_only`. | `verifica-repo.sh` (ricerca nel codice); prove Rust |
| S7 | Un solo `Log` (in `Contenitore`), nessun `println`/`printStackTrace`. Nel registro mai dati personali. | `verifica-repo.sh`; prova Rust `registro_senza_dati_personali` |
| S8 | `FLAG_SECURE` in `release` e `collaudo`: niente schermate né anteprima nelle app recenti. | revisione; collaudo C12 |
| S9 | **Appunti**: `ClipData` con etichetta `cup-piemonte` e l'extra `android.content.extra.IS_SENSITIVE = true`. Dopo 120 secondi, se l'app è in primo piano e l'etichetta degli appunti è ancora `cup-piemonte`, `clearPrimaryClip()`; se non è in primo piano, lo stesso controllo al prossimo `onResume`. | prova strumentata; collaudo C11 |
| S10 | Le notifiche con dati della visita sono `VISIBILITY_PRIVATE` con una versione pubblica senza dati. | prova strumentata sul `Notification` costruito |
| S11 | L'app non legge extra né dati dagli intent in ingresso: contano solo le due azioni `cup.GIRO` e `cup.FERMA`, e i ricevitori ignorano ogni azione non prevista. | revisione; prova strumentata con un'azione sconosciuta |
| S12 | Ogni `PendingIntent` è `FLAG_IMMUTABLE` e con intent esplicito. | lint (`UnspecifiedImmutableFlag` fatale); revisione |
| S13 | `filterTouchesWhenObscured` sulla finestra. Sposta, prenota e disdici partono solo dal riquadro di conferma con data, ora e luogo. | revisione; prove di `StatoApp` |
| S14 | L'APK di rilascio è firmato (schemi v2 e v3) con il certificato la cui impronta è in `android/firma/impronta-sha256.txt`; non è `debuggable`. | `verifica-apk.sh` |
| S15 | L'app non scarica e non installa codice: niente `REQUEST_INSTALL_PACKAGES`, niente `DexClassLoader`, dal canale delle versioni si prende solo un numero. | `verifica-apk.sh`, `verifica-repo.sh`, prove Rust |
| S16 | Ogni APK pubblicato ha `SHA256SUMS.txt` e un'attestazione di provenienza (`actions/attest-build-provenance`). | workflow |
| S17 | Dipendenze Kotlin solo quelle del capitolo 5; Rust con `Cargo.lock` e `--locked`. | `verifica-repo.sh` confronta `libs.versions.toml` con l'elenco ammesso; CI |
| S18 | `cargo audit` senza vulnerabilità note; Dependabot acceso per `cargo` e `gradle` (solo avvisi di sicurezza). | CI; U7 |
| S19 | Nessun segreto nel repository: niente `*.jks`, `*.p12`, `*.keystore`, `keystore.properties`, `.env` tra i file tracciati; nessuna stringa `nsec1`, `BEGIN PRIVATE KEY`, `storePassword` con un valore. | `verifica-repo.sh`; secret scanning (U7) |
| S20 | Il job che firma usa solo action `actions/*`, gira nell'ambiente `rilascio-android` e ha `permissions: contents: read`. Le action di terze parti degli altri job sono fissate a uno SHA (fase 8). | revisione dei workflow |
| S21 | R8 acceso in `release`; le librerie native sono solo `libcup_mobile.so` e `libjnidispatch.so` per `arm64-v8a` e `armeabi-v7a`, allineate a 16 KB su arm64. | `verifica-apk.sh` |
| S22 | Una sola istanza del motore; mai due controlli insieme sulla stessa ricetta. | prova Rust `seconda_apertura_rifiutata`; prova strumentata: due `Contenitore.motore()` danno lo stesso oggetto |

**`ChiaveArchivio`** (in `shared/androidMain`), interfaccia:

```kotlin
object ChiaveArchivio {
    sealed interface Esito {
        class Pronta(val esadecimale: String) : Esito
        data object Mancante : Esito                       // il file chiave.protetta non c'è
        class Illeggibile(val motivo: String) : Esito      // c'è ma non si decifra (motivo: nome dell'eccezione)
    }
    fun leggi(cartella: File): Esito
    fun crea(cartella: File): String                       // sostituisce alias e file; ritorna l'esadecimale
}
```

Chiave del Keystore: `KeyGenParameterSpec.Builder(ALIAS, PURPOSE_ENCRYPT or PURPOSE_DECRYPT)` con
`setBlockModes(BLOCK_MODE_GCM)`, `setEncryptionPaddings(ENCRYPTION_PADDING_NONE)`, `setKeySize(256)`. Cifratura
`AES/GCM/NoPadding`, IV scelto dal Keystore (`cipher.iv`), decifratura con `GCMParameterSpec(128, iv)`.

### 10.3 Verifiche automatiche

- **`verifica-repo.sh`** (a ogni PR): regola 2 (file toccati rispetto a `origin/master`); S6, S7, S15 (ricerche nel
  codice Kotlin); S17; S19; `versione.properties` e `rust/Cargo.toml` con la stessa versione.
- **Lint di Android** su `release`, con `abortOnError` e questi controlli portati a **fatali**:
  `UnspecifiedImmutableFlag`, `UnsafeIntentLaunch`, `ExportedReceiver`, `ExportedService`, `ExportedContentProvider`,
  `AllowBackup`, `HardcodedDebugMode`, `SetJavaScriptEnabled`, `TrustAllX509TrustManager`, `BadHostnameVerifier`,
  `WorldReadableFiles`, `WorldWriteableFiles`, `UnprotectedSMSBroadcastReceiver`, `InsecureBaseConfiguration`,
  `MissingPermission`, `ScheduleExactAlarm`, `ForegroundServiceType`. Niente `baseline`, niente `disable`. Se lint
  dice che uno di questi nomi non esiste nella sua versione, lo si toglie dall'elenco e lo si scrive nel riporto
  della fase.
- **Clippy** con `-D warnings` e **`cargo audit`** sulla crate.
- **`verifica-apk.sh <apk> <tipo>`** (`rilascio` o `prova`), con `aapt2`, `apksigner`, `zipalign` e
  `llvm-readelf` dell'SDK: S1, S2, S3, S14, S15, S21; nome del pacchetto, `versionName` e `versionCode` attesi;
  `minSdk` 26 e `targetSdk` 35; `zipalign -c -P 16 -v 4`; nessuna delle stringhe di S19 dentro l'APK; dimensione
  sotto i 40 MB. Per `prova`: id con `.prova`, ABI con anche `x86_64`.
- **Prove strumentate** sull'emulatore (API 34, x86_64): quelle citate in 10.2, più `versione_motore()` che
  risponde, `ora_locale()` uguale all'ora di `java.time` (il fuso arriva al Rust), apri → salva → riapri.
- **`prova-avvio.sh`**: installa l'APK `collaudo` sull'emulatore, lo avvia, aspetta fino a 60 secondi
  `CUP: avvio completato` in `logcat`; fallisce se compare `FATAL EXCEPTION` o `UnsatisfiedLinkError`. Prova
  l'APK passato da R8, cioè quello che più somiglia al rilascio.
- **`mobsfscan`** sul codice Kotlin: il rapporto si allega come artifact; non blocca. Si legge nella fase 8.

### 10.4 Revisione di sicurezza (fase 8)

La fa un modello più capace di quello che ha implementato, su tutto `android/` e sui workflow:

1. La lista di 10.2, requisito per requisito, con l'esito e dove lo si è visto → `android/SICUREZZA.md`.
2. Lettura intera di: `ChiaveArchivio`, `Contenitore`, `ServizioControlli`, i due ricevitori, `Notifiche`,
   `MotoreRust`, `comandi.rs`, `portachiavi.rs`, `versione.rs`, il manifest, i quattro workflow, gli script.
3. Confronto riga per riga di `comandi.rs` con `main.rs`: ogni controllo dell'originale (ricetta doppia,
   operazione in corso, esito incerto, 20 minuti dopo l'eliminazione, `automatica_prima`) c'è ancora.
4. Il rapporto di `mobsfscan`: ogni voce o si corregge o si spiega perché non vale.
5. Fissare a uno SHA le action di terze parti (`git ls-remote https://github.com/<proprietario>/<action>
   refs/tags/<tag>`), con il tag in un commento.
6. Sul telefono o sull'emulatore, con l'APK `collaudo`: `adb shell dumpsys package <id>` (permessi e componenti),
   `adb logcat` durante un uso completo con dati finti (nessun dato personale), `adb backup`/trasferimento che non
   porta fuori nulla, `run-as` rifiutato.
7. Ogni problema trovato si corregge in quella fase; quelli accettati si scrivono in `SICUREZZA.md` con il motivo
   e li approva l'utente.

## 11. Build, verifiche e rilascio

### 11.1 `android-prove.yml`

Parte su `pull_request` e su `push` a `master` per i percorsi `android/**`, `app/core/**`, `app/desktop/src/**`,
`.github/workflows/android-*.yml`, e con `workflow_call`. `permissions: contents: read`. Quattro job su
`ubuntu-24.04`:

| Job | Passi |
|---|---|
| `repo` | checkout con `fetch-depth: 0`; `android/scripts/verifica-repo.sh` |
| `rust` | `cargo test --locked`; `cargo clippy --all-targets --locked -- -D warnings`; `cargo install cargo-audit --locked` e `cargo audit` (tutto da `android/rust`) |
| `android` | `actions/setup-java` (Temurin 17); `rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android`; `cargo install cargo-ndk --version 3.5.4 --locked`; `sdkmanager "ndk;28.2.13676358"`; `gradle/actions/wrapper-validation`; `gradle/actions/setup-gradle`; `./gradlew :shared:testDebugUnitTest :androidApp:lintRelease :androidApp:assembleDebug :androidApp:assembleCollaudo`; `verifica-apk.sh … prova` sull'APK `collaudo`; `pip install mobsfscan` e `mobsfscan --json` su `android/`; artifact: APK `debug`, APK `collaudo`, rapporti di lint e di mobsfscan |
| `emulatore` | dopo `android`; abilita KVM; `reactivecircus/android-emulator-runner` (API 34, x86_64); `./gradlew :androidApp:connectedDebugAndroidTest`; `prova-avvio.sh` sull'APK `collaudo` |

Le action di terze parti sono solo queste: `gradle/actions`, `reactivecircus/android-emulator-runner` e
`Swatinem/rust-cache` (cache del Rust, come nei workflow dell'app desktop). Le altre sono `actions/*`.

Qui non ci sono segreti: `collaudo` si firma con la chiave di debug e `lintRelease` non ha bisogno della firma.

### 11.2 `android-apk.yml`

Parte con un tag `android-v*` oppure a mano (`workflow_dispatch`, ingresso `tipo`: `prova`, il solo valore).

1. `versione`: il tag (se c'è) è `android-v` + `versione` di `versione.properties` + uguale a `rust/Cargo.toml`.
2. `prove`: `uses: ./.github/workflows/android-prove.yml`.
3. `apk` (`environment: rilascio-android`, `permissions: contents: read, id-token: write, attestations: write`,
   solo action `actions/*`; Rust, cargo-ndk e NDK si installano con `rustup`, `cargo install` e `sdkmanager`, senza
   cache): scrive il keystore da `ANDROID_KEYSTORE_BASE64` in un file fuori dal checkout
   (`$RUNNER_TEMP`), `./gradlew :androidApp:assembleRelease` (o `assembleCollaudo` per `prova`), cancella il
   keystore, rinomina in `CUP-Piemonte_<versione>_android.apk` (per `prova`: `…_android-prova.apk`),
   `verifica-apk.sh`, `sha256sum` in `SHA256SUMS.txt`, attestazione di provenienza, artifact.
4. `bozza` (solo con il tag; `permissions: contents: write`): crea una Release **in bozza**, segnata come
   pre-release, con APK, `SHA256SUMS.txt` e le note di `note-release.sh` (cosa fa l'app, come installarla,
   l'avviso di Play Protect, l'impronta del certificato, le novità).

Il nome dell'APK non contiene nomi di ABI (Zapstore scarterebbe un file con `armeabi-v7a` nel nome).

### 11.3 `android-canale.yml`

Parte quando una Release con tag `android-v*` viene **pubblicata**, o a mano con un tag (per tornare alla versione
prima). Scarica l'APK dalla Release, rifà `verifica-apk.sh` e scrive `latest.json`
(`{"versione": "0.1.0", "codice": 100}`, generato da `canale.py`) nella Release `android-aggiornamenti`
(la crea se manca, come pre-release e non "latest"), con `gh release upload --clobber`; poi lo rilegge dall'indirizzo
pubblico e lo confronta.

### 11.4 Pubblicare una versione

1. Alzare la versione in `android/versione.properties` e in `android/rust/Cargo.toml`, `cargo build` per
   `Cargo.lock`, PR, merge.
2. Tag: `git tag -a android-vX.Y.Z -m "App Android X.Y.Z"` e push del tag.
3. L'utente approva il job `apk`; nasce la Release in bozza.
4. L'utente installa l'APK della bozza, fa il [collaudo](#13-collaudo-sul-telefono) e pubblica.
5. `android-canale.yml` aggiorna il canale: le app installate avvisano entro 24 ore.

Per un **APK di prova** senza Release: avviare a mano `android-apk.yml` sul ramo, approvare, scaricare l'artifact.
Si installa accanto all'app vera (id `.prova`) e ogni prova aggiorna la precedente, perché la firma è la stessa.

## 12. Fasi

Ogni fase: una PR verso `master`. "Locale" = si verifica in sessione; "CI" = serve la CI (o U2).

| Fase | Cosa | Chi | Stato |
|---|---|---|---|
| 0 | Analisi e questo piano | — | fatta |
| 1 | Crate Rust con l'interfaccia per Kotlin | modello base, con revisione | da fare |
| 2 | Progetto Gradle, CI, primo APK che si installa | modello base | da fare |
| 3 | Chiave nel Keystore e apertura dell'archivio | modello base | da fare |
| 4 | Interfaccia | modello base | da fare |
| 5 | Servizio, sveglia, notifiche | modello base, con revisione | da fare |
| 6 | Avviso di versione nuova, impostazioni | modello base | da fare |
| 7 | Firma, Release, canale delle versioni | modello base | da fare |
| 8 | Revisione di sicurezza | modello più capace | da fare |
| 9 | Collaudo sul telefono e versione 0.1.0 | utente | da fare |
| 10 | Zapstore | utente e modello base | da fare |

Chi chiude una fase aggiorna la colonna "Stato" nella stessa PR.

### Fase 1 — La crate Rust (difficoltà media; verifica locale)

**Fa**: tutto il capitolo 6 e `android/scripts/controllo-android.sh`. Solo il job `rust` e il job `repo` di
`android-prove.yml` (gli altri due arrivano nella fase 2), `verifica-repo.sh` con i controlli che hanno già senso.

**Accettazione**
- `cargo test --locked`: passano le 44 prove dei moduli inclusi e quelle di 6.5.
- `cargo clippy --all-targets --locked -- -D warnings` pulito.
- `controllo-android.sh` termina con `Finished` per `aarch64-linux-android` e `armv7-linux-androideabi`.
- Il generatore produce `uniffi/cup_mobile/cup_mobile.kt` con `class Motore`, `interface Ascoltatore`,
  `sealed class Sbaglio` e i tipi di 6.2.
- `git diff --stat origin/master -- app/ telegram-bot/ docs/` è vuoto.

**Revisione**: un modello più capace confronta `comandi.rs` con `main.rs` (punto 3 di 10.4) prima del merge.

### Fase 2 — Scheletro Gradle e CI (difficoltà alta per la messa a punto; CI)

**Serve**: U1, e conviene U2. **Fa**: capitolo 7; `shared` con il solo `MotoreRust.versioneMotore()`;
`androidApp` con manifest (senza servizio e ricevitori), `App`, `MainActivity` che mostra "CUP Piemonte · motore
X.Y.Z"; icone (9.8); i job `android` ed `emulatore`; `verifica-apk.sh` (i controlli che hanno già senso, tipo
`prova`); `prova-avvio.sh`; `prepara-ambiente.sh` (installa cmdline-tools, piattaforma, build-tools e NDK in
`$ANDROID_HOME` per chi lavora in sessione).

**Accettazione**
- La CI è verde su tutti e quattro i job.
- L'APK `debug` dell'artifact si installa su un telefono vero (lo prova l'utente) e mostra la versione del motore:
  la libreria Rust si carica su arm64.
- Sull'emulatore: una prova strumentata chiama `versione_motore()`; `prova-avvio.sh` passa sull'APK `collaudo`
  (R8 non rompe JNA).
- `verifica-apk.sh` trova le due librerie native per ogni ABI e l'allineamento a 16 KB.

### Fase 3 — Chiave, apertura dell'archivio, `MotoreRust` (difficoltà media; CI)

**Fa**: `ChiaveArchivio`, `Contenitore`, `MotoreRust` completo con le conversioni, il modello e le interfacce di
8.1, la schermata di errore di avvio. **Accettazione**: prove strumentate di S3, S4, S22, apri → salva → riapri
dopo aver ricreato il `Contenitore`, `ora_locale()`; prova di ogni conversione `…M` → modello.

### Fase 4 — Interfaccia (difficoltà media; locale per `commonTest`, CI per il resto)

**Fa**: 8.3, 8.4, 8.5, `PiattaformaAndroid` (appunti compresi, S9), `MainActivity` completa (9.7). Il servizio
ancora non c'è: i controlli non partono da soli. Per poter provare, una voce **solo nel tipo `debug`**
"Controlla adesso" in Impostazioni che chiama `giro()`.

**Accettazione**: le prove di 8.5 passano; sull'emulatore, con un `MotoreFinto` iniettato, una prova di
interfaccia aggiunge una ricetta, apre il riquadro "Spostare la prenotazione?" e annulla; l'utente, sul telefono,
aggiunge una ricetta vera e vede prenotazione, zona e giorni (collaudo C1-C3).

### Fase 5 — Background (difficoltà alta; CI e telefono)

**Fa**: 9.3, 9.4, 9.5, il Benvenuto con i permessi, i riquadri di avviso della schermata principale, le righe del
registro di 9.7. Toglie "Controlla adesso".

**Accettazione**: prove strumentate: il servizio parte ed espone la notifica fissa; `cup.FERMA` lo ferma e mette
`controlli_fermi`; un'azione sconosciuta non fa nulla (S11); le notifiche hanno la versione pubblica (S10);
`Sveglia.programma` registra un allarme (`dumpsys alarm` contiene il pacchetto). Collaudo C4-C9 dell'utente.

**Revisione**: un modello più capace rilegge servizio, sveglia e ricevitori contro 9.3 e 9.4.

### Fase 6 — Versione nuova e impostazioni (difficoltà bassa)

**Fa**: 9.6 e la schermata Impostazioni completa. **Accettazione**: prove Rust di 6.4; con `latest.json` di prova
servito da un indirizzo `https://github.com/…` (la Release `android-aggiornamenti` creata a mano con una versione
più alta) "Controlla ora" dice "C'è la versione X." e il pulsante apre la pagina giusta.

### Fase 7 — Rilascio (difficoltà media; CI)

**Serve**: U3, U4, U5, U6. **Fa**: firma `release`, `android-apk.yml`, `android-canale.yml`, `note-release.sh`,
`canale.py`, `verifica-apk.sh` completo, `android/README.md` (cosa fa, come si installa, permessi e perché, dove
stanno i dati, come si disinstalla, limiti noti), la sezione "App Android" in fondo al `README.md` del repository.

**Accettazione**: avviato a mano, `android-apk.yml` produce `CUP-Piemonte_<v>_android-prova.apk` firmato con il
keystore vero, e `verifica-apk.sh` passa (impronta compresa); l'utente lo installa e un secondo APK di prova lo
aggiorna senza disinstallare.

### Fase 8 — Sicurezza (modello più capace)

**Serve**: U7. **Fa**: 10.4 per intero, `.github/dependabot.yml`, `android/SICUREZZA.md`.
**Accettazione**: ogni riga di 10.2 ha esito "verificato" con il riferimento; nessun problema aperto che l'utente
non abbia accettato per iscritto.

### Fase 9 — Collaudo e prima versione

L'utente fa il capitolo 13 con un APK di prova per almeno 48 ore. I problemi si correggono in PR piccole. Poi tag
`android-v0.1.0` e 11.4.

### Fase 10 — Zapstore

Capitolo 14.

## 13. Collaudo sul telefono

Lo fa l'utente, con l'APK di prova e una ricetta vera. I passi segnati **(vero)** cambiano una prenotazione vera.

| # | Prova | Esito atteso |
|---|---|---|
| C1 | Installare, aprire, fare il Benvenuto | I tre permessi risultano ✓ |
| C2 | Aggiungere una ricetta già prenotata | Data, ora, luogo e prestazione giusti; CF e ricetta mascherati |
| C3 | Aggiungere una ricetta con un numero sbagliato | Messaggio d'errore, nessuna ricetta salvata |
| C4 | Lasciare l'app chiusa (tolta dalle recenti) per 2 ore, schermo spento | Nel registro un controllo circa ogni 20 minuti; notifica fissa sempre presente |
| C5 | Una notte intera, telefono non in carica | Come C4; annotare gli intervalli più lunghi |
| C6 | Riavviare il telefono e non aprire l'app | Dopo lo sblocco torna la notifica fissa e i controlli riprendono |
| C7 | "Ferma i controlli" dalla notifica, poi "Riprendi" dall'app | Si fermano davvero; riprendono |
| C8 | Quando esce una data: la notifica | Data, ora e luogo; a telefono bloccato solo "C'è una novità" |
| C9 | **(vero)** "Sposta qui" → "Sì, sposta" | Spostata; l'app lo conferma dopo la verifica; sul sito del CUP risulta |
| C10 | **(vero, se serve)** "Disdici" → "Sì, disdici" | Disdetta; ricetta in pausa come sul computer |
| C11 | "Copia" su codice fiscale, incollare altrove, aspettare 2 minuti con l'app aperta | Incolla il valore intero; dopo 2 minuti gli appunti sono vuoti |
| C12 | Fare una schermata dell'app; guardare l'anteprima nelle app recenti | Schermata rifiutata; anteprima vuota |
| C13 | Modalità aereo durante un controllo | Errore nella scheda, nessun blocco; al controllo dopo riprende |
| C14 | Installare un secondo APK di prova sopra il primo | Le ricette restano; i controlli riprendono da soli |
| C15 | Disinstallare e reinstallare | Si riparte da zero, senza errori |

Se C4 o C5 mostrano buchi lunghi (più di un'ora): riportare marca, modello, versione di Android e le righe
`sveglia:` e `servizio:` del registro. Il rimedio (istruzioni per marca, o altro) si decide con quei dati.

## 14. Zapstore

Fatti verificati leggendo `zsp` (lo strumento di pubblicazione di Zapstore, versione 2):

- prende l'APK dalla Release di GitHub; accetta un APK con più ABI purché ci sia `arm64-v8a`; scarta i file con
  `armeabi-v7a`, `x86`, `x86_64` nel nome;
- richiede una **prova che il certificato di firma appartiene alla chiave Nostr** di chi pubblica: si crea con la
  procedura guidata `zsp`, **a mano, dall'utente**, con il keystore. Chi implementa non riceve mai keystore,
  password o `nsec`;
- per pubblicare serve `SIGN_WITH` (un `nsec` o, meglio, un indirizzo `bunker://`).

**L'utente (U9)**: installa `zsp`; crea o sceglie la chiave Nostr; da `android/`, con la prima Release già
pubblicata, lancia `zsp` e segue la procedura (prova del certificato compresa); crea l'ambiente GitHub `zapstore`
con approvazione e il segreto `ZAPSTORE_SIGN_WITH`.

**Chi implementa**: `android/zapstore.yaml` (se la procedura guidata l'ha già scritto, si tengono le sue righe
`repository`, `release_source` e `metadata_sources`):

```yaml
repository: https://github.com/francescopelle/cup-piemonte-checker
release_filter: '^android-v[0-9]+\.[0-9]+\.[0-9]+$'
match: '_android\.apk$'
name: CUP Piemonte (non ufficiale)
summary: Avvisa quando esce una data prima per la tua visita sul CUP Piemonte
description: |
  Controlla il portale del CUP Piemonte ogni 20 minuti e avvisa quando si libera una data prima della
  prenotazione. Sposta, prenota o disdice solo quando lo chiedi. Non ufficiale: non è collegata alla
  Regione Piemonte né al CSI Piemonte. I dati restano sul telefono, cifrati.
license: MIT
tags: [health, utilities]
icon: ./androidApp/src/main/ic_launcher-512.png
release_notes: ./NOVITA.md
```

e `android-zapstore.yml`: solo a mano, `environment: zapstore`, scarica `zsp` a una versione fissata verificandone
la somma SHA-256, lancia `zsp publish -q` da `android/` e riporta l'esito (`published`, `pending` o il motivo del
rifiuto). `android/NOVITA.md` tiene le novità per versione (formato "Keep a Changelog") e lo usa anche
`note-release.sh`.

**Accettazione**: `zsp publish` risponde `published` o `pending`; l'app compare in Zapstore e si installa da lì
sopra quella installata dall'APK (stessa firma).

## 15. Manutenzione

- **Prendere le novità dell'app desktop**: `git fetch` dal repository di origine e merge in `master`. I file di
  `app/` non sono stati toccati, quindi non ci sono conflitti. `android-prove.yml` parte anche per `app/core/**` e
  `app/desktop/src/**`: se un modulo incluso non compila più nella crate Android, la CI lo dice subito.
- **Dopo ogni merge**: `git diff <prima>..<dopo> -- app/desktop/src/main.rs app/desktop/ui/`. Ogni cambiamento ai
  comandi va riportato in `comandi.rs`, ogni cambiamento a testi e regole dell'interfaccia in `shared`. Se
  `controlli::avvia` cambia, si rivede `giro()`.
- **Se `segnalazioni.rs` o un altro modulo incluso comincia a usare qualcosa di `crate::` che la crate Android non
  ha**: si aggiunge in `android/rust/src` una copia minima con lo stesso nome, come `aggiornamenti.rs`.
- **Dipendenze**: gli avvisi di Dependabot e di `cargo audit` si chiudono alzando la versione nella stessa minor;
  un salto di major è una decisione dell'utente.

## 16. Fuori dal piano

Non si fanno ora; qui per non perderli.

- **Server che fa i controlli** al posto del telefono (idea dell'utente per dopo). L'interfaccia `Motore` in
  `commonMain` è il punto di innesto: una seconda implementazione che parla con un server non tocca le schermate.
  Cambia il modello di riservatezza (codice fiscale e ricetta escono dal telefono) e va progettato a parte.
- **iOS**. Il Rust compila anche per iOS e `commonMain` è già separato; mancano i binding (Swift o Gobley), il
  target KMP e un modo di lavorare in background, che su iOS è molto più limitato.
- Blocco dell'app con impronta o PIN. Aggiornamento dell'APK dentro l'app. Segnalazioni dei problemi. Play Store.
  Completamento automatico dei comuni. Lingue diverse dall'italiano. Tablet e schermi grandi.

## Appendice A: file verificati

Compilati e provati così (45 prove passate, clippy pulito, `cargo check` per `aarch64-linux-android` e
`armv7-linux-androideabi`, binding Kotlin generati con uniffi 0.32.2).

**`android/rust/Cargo.toml`**

```toml
[package]
name = "cup-mobile"
description = "Ponte tra il motore dell'app e l'app Android (UniFFI)"
version = "0.1.0"
edition = "2021"
license = "MIT"
publish = false

[lib]
crate-type = ["cdylib", "lib"]
name = "cup_mobile"

[[bin]]
name = "uniffi-bindgen"
path = "src/bin/uniffi-bindgen.rs"
required-features = ["bindgen"]

[features]
bindgen = ["uniffi/cli"]

[dependencies]
chacha20poly1305 = "0.10"
chrono = { version = "0.4", default-features = false, features = ["clock", "std"] }
cup-core = { path = "../../app/core" }
keyring = { version = "3", default-features = false }
regex = "1"
reqwest = { version = "0.12", default-features = false, features = ["blocking", "rustls-tls"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
uniffi = "=0.32.2"

# una crate a se', fuori dal workspace di app/: Cargo.toml e Cargo.lock dell'app desktop non cambiano
[workspace]

[profile.release]
codegen-units = 1
lto = true
opt-level = "s"
strip = true
panic = "unwind"
```

**`android/rust/src/bin/uniffi-bindgen.rs`**

```rust
fn main() {
    uniffi::uniffi_bindgen_main()
}
```

**Inizio di `android/rust/src/lib.rs`**

```rust
//! Il ponte tra l'app Android e il motore. I moduli dell'app desktop si includono cosi' come sono: non si copiano
//! e non si modificano (vedi android/PIANO.md, decisione D3).
#![allow(dead_code)]

#[path = "../../../app/desktop/src/archivio.rs"]
mod archivio;
#[path = "../../../app/desktop/src/controlli.rs"]
mod controlli;
#[path = "../../../app/desktop/src/operazioni.rs"]
mod operazioni;
#[path = "../../../app/desktop/src/registro.rs"]
mod registro;
#[path = "../../../app/desktop/src/segnalazioni.rs"]
mod segnalazioni;

mod aggiornamenti;
mod comandi;
mod portachiavi;
mod tipi;
mod versione;

uniffi::setup_scaffolding!();
```

**`android/rust/src/portachiavi.rs`**

```rust
//! Il "portachiavi" su Android: la chiave arriva da Kotlin (Android Keystore) e qui si restituisce soltanto.
//! Senza questo, `keyring` su Android userebbe un portachiavi finto che non conserva nulla.
use std::any::Any;
use std::sync::Mutex;

use keyring::credential::{Credential, CredentialApi, CredentialBuilderApi, CredentialPersistence};

static CHIAVE: Mutex<Option<String>> = Mutex::new(None);

struct Voce;

impl CredentialApi for Voce {
    fn set_secret(&self, _: &[u8]) -> keyring::Result<()> {
        Err(keyring::Error::NoStorageAccess("la chiave la conserva Android".into()))
    }
    fn get_secret(&self) -> keyring::Result<Vec<u8>> {
        match CHIAVE.lock().unwrap_or_else(|e| e.into_inner()).clone() {
            Some(k) => Ok(k.into_bytes()),
            None => Err(keyring::Error::NoStorageAccess("chiave non impostata".into())),
        }
    }
    fn delete_credential(&self) -> keyring::Result<()> {
        Err(keyring::Error::NoStorageAccess("la chiave la conserva Android".into()))
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}

struct Costruttore;

impl CredentialBuilderApi for Costruttore {
    fn build(&self, _: Option<&str>, _: &str, _: &str) -> keyring::Result<Box<Credential>> {
        Ok(Box::new(Voce))
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn persistence(&self) -> CredentialPersistence {
        CredentialPersistence::UntilDelete
    }
}

/// Da chiamare subito prima di `Archivio::apri`, ogni volta.
pub fn installa(chiave_esadecimale: String) {
    *CHIAVE.lock().unwrap_or_else(|e| e.into_inner()) = Some(chiave_esadecimale);
    keyring::set_default_credential_builder(Box::new(Costruttore));
}
```

**La prova della chiave** (da adattare alla firma definitiva di `apri`): apre una cartella vuota con la chiave
`"ab".repeat(32)`, salva `prossimo_id = 7`, chiude; riapre con la stessa chiave e ritrova 7, e `chiave.segreta` non
esiste; riapre con `"cd".repeat(32)` e trova un archivio vuoto con `messo_da_parte` valorizzato.

## Appendice B: controllo per Android senza NDK

`android/scripts/controllo-android.sh` verifica che la crate compili per Android dove l'NDK non c'è (basta
`clang`). È un controllo dei tipi, non una build: la libreria vera la costruisce la CI con l'NDK. `ring` vuole due
header C che senza NDK mancano: lo script li crea finti in una cartella temporanea.

```bash
#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../rust"
finti=$(mktemp -d)
printf '#pragma once\n#define assert(x) ((void)0)\n#define static_assert _Static_assert\n' > "$finti/assert.h"
printf '#pragma once\n#include <stddef.h>\nvoid *memcpy(void *, const void *, size_t);\nvoid *memmove(void *, const void *, size_t);\nvoid *memset(void *, int, size_t);\nint memcmp(const void *, const void *, size_t);\nsize_t strlen(const char *);\n' > "$finti/string.h"
clang_inc=$(clang -print-resource-dir)/include
ar=$(command -v llvm-ar || command -v llvm-ar-18)
for coppia in aarch64-linux-android:aarch64-linux-android26 armv7-linux-androideabi:armv7a-linux-androideabi26; do
  rust=${coppia%%:*}; c=${coppia##*:}; var=${rust//-/_}
  rustup target add "$rust" >/dev/null
  env "CC_$var=clang" "AR_$var=$ar" \
      "CFLAGS_$var=--target=$c -ffreestanding -nostdinc -isystem $clang_inc -isystem $finti" \
      cargo check --locked --lib --target "$rust"
done
```
