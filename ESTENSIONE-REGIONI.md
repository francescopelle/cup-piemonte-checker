# Estensione ad altre regioni: analisi e piano

Stato: novembre 2026. Pilota: **Toscana** e **Calabria**. Poi le altre regioni, una alla volta.

> **Come leggere questo documento.** Il codice del Piemonte è stato letto direttamente. I portali delle altre
> regioni **non sono stati aperti**: dall'ambiente in cui è stata fatta l'analisi la rete li blocca, quindi
> tutto ciò che riguarda Toscana, Calabria e le altre regioni viene da estratti dei motori di ricerca, da
> documenti ufficiali citati e dal codice di progetti di terzi su GitHub. Ogni affermazione porta un segno:
> **[V]** verificato su codice o fonte letta direttamente, **[R]** riportato da una fonte non aperta
> direttamente, **[?]** da verificare dal vivo. La prima fase del piano serve proprio a togliere i [?].

## 1. In breve

- **La tecnica del Piemonte è riusabile, il codice del portale no.** Codice fiscale + numero ricetta senza
  login risulta possibile sia in Toscana sia in Calabria, e in entrambe esistono online lettura della
  prenotazione, spostamento e disdetta [R]. Ma ogni portale è un prodotto diverso: va scritto un modulo per
  regione, e va separato ciò che oggi nel motore è "Piemonte" da ciò che è generale (zone, calendario,
  verifiche, archivio, notifiche).
- **Toscana è il candidato più facile da leggere e il più delicato da usare.** Il portale espone API JSON
  semplici, senza stato di sessione né captcha [V su codice di terzi]: niente moduli JSF da replicare. In
  cambio servono le **ultime cifre della tessera sanitaria** oltre a codice fiscale e ricetta, e **ogni
  ricerca riserva davvero i posti che mostra** (uno per struttura), togliendoli agli altri per alcuni minuti.
  Prima di qualsiasi controllo periodico va trovata la chiamata che li rilascia.
- **Calabria è plausibile ma tutta da esplorare.** Il portale regionale accetta codice fiscale + ricetta e ha
  "Recupera prenotazioni" con modifica e annullo [R]; tecnologia, chiamate e captcha sono ignoti [?], e non
  esistono progetti di terzi da cui partire. Ha però un'area pubblica "Prime disponibilità" che permette un
  avviso anche senza identificarsi.
- **Si parte in sola lettura.** Per ogni regione nuova: prima "ti avviso quando esce una data prima", poi
  "sposta con il tuo Sì", per ultima la prenotazione automatica. È lo stesso ordine con cui è cresciuto il
  Piemonte, e ogni passo si sblocca solo dopo prove dal vivo con una ricetta vera.
- **Servono due cose che il codice non può dare**: l'accesso di rete ai due portali (per leggerne le pagine
  e gli script) e almeno una ricetta vera per regione, meglio se già prenotata, per registrare le chiamate
  di lettura, spostamento e disdetta.

## 2. Com'è fatto oggi (Piemonte)

Tre progetti separati, senza codice in comune: l'app (`app/`, Rust: `core` + `cli` + `desktop` Tauri), il
bot Telegram (`telegram-bot/`, Python) e lo script originale. Il motore dell'app è `app/core`:

| File | Cosa fa | Quanto è legato al Piemonte |
|---|---|---|
| `portale.rs` | La sessione: cookie, moduli JSF/ICEfaces, POST replicati campo per campo | Tutto: indirizzo, pagine, nomi dei moduli, "Sposta", "Estendi area" |
| `analisi.rs` | Legge le pagine: date, luoghi, elenco, Riepilogo | Quasi tutto (espressioni regolari sull'HTML del portale); generali solo i tipi `Luogo`, `Prenotazione`, `Slot` e il controllo di codice fiscale e ricetta |
| `operazioni.rs` | Sposta, prenota, disdice, con verifica a sessione nuova | La logica di sicurezza è generale; i passi (Riepilogo, presa visione, Conferma) sono del portale; il numero del call center è fisso |
| `zona.rs` | Dove va bene una data: sede, comune, provincia, giorni | Le regole sono generali; i dati no: comuni del Piemonte, 8 province, "Torino e prima cintura", il modo in cui il portale scrive l'indirizzo |
| `distanza.rs` | Distanza delle sedi dal comune di partenza | Generale, sopra i dati dei comuni |
| `contatti.rs` | Email e cellulare per le conferme | Generale |

Fuori dal motore sono piemontesi: il nome dell'app e degli installer (`tauri.conf.json`, i workflow), la
scelta "Torino e prima cintura" nell'interfaccia, i testi, e nel bot le impronte delle pagine
(`impronte.py`) e la sonda (`sonda.py`).

Le caratteristiche del portale piemontese che hanno dato forma al motore:

1. **Accesso anonimo** con codice fiscale + NRE, nessun captcha attivo.
2. **Applicazione con stato sul server** (JSF/ICEfaces su Liferay): ogni click è un POST che deve portare lo
   stato della pagina. È la parte più fragile e più lunga da scrivere.
3. **"Sposta appuntamento" è una funzione del portale**: si sceglie la data nuova e il portale sostituisce la
   vecchia in un passo solo. Non si resta mai senza appuntamento.
4. **Aprire "Sposta" blocca la data proposta per circa 40 minuti**, senza modo di rilasciarla: da qui
   l'intervallo minimo di 20 minuti tra due controlli.
5. Le date lontane compaiono solo premendo "Estendi area di ricerca".

Ognuno di questi cinque punti va ricontrollato in ogni regione: sono le domande della sezione 5.

## 3. Toscana

**Portale**: `https://prenota.sanita.toscana.it/`, lo sportello per i cittadini del CUP 2.0 regionale, unico
per tutte le aziende, online in questa forma dal luglio 2023; manutenzione di Engineering [R]. Non passano
dal portale: analisi di laboratorio (`zerocode.sanita.toscana.it`), ricette con priorità U, libera
professione [R].

| | Piemonte | Toscana |
|---|---|---|
| Accesso anonimo | CF + NRE | CF + NRE + **ultime cifre della tessera sanitaria** (parametro `numbersTeam`) [V su codice di terzi]; quante cifre: [?] |
| Tecnologia | JSF/ICEfaces, stato sul server | **API REST/JSON**, nessun cookie di sessione, nessun token [V su codice di terzi] |
| Captcha / anti-bot | disattivato | nessuna traccia nel client di terzi [V]; da ricontrollare sulla pagina [?] |
| Lettura della prenotazione | "Recupera Prenotazioni" | sezione "I tuoi appuntamenti" con CF + NRE [R]; chiamata: [?] |
| Spostamento | sempre, in un passo | **solo per le prestazioni che lo consentono**, altrimenti disdetta e nuova prenotazione [R]; chiamata: [?] |
| Disdetta | sì | sì, fino a 48 ore prima; oltre, solo al telefono [R]; chiamata: [?] |
| Ricerca delle date | proposta + "Altre disponibilità" + "Estendi area" | una chiamata: **un appuntamento per struttura**, nell'area dell'assistito (`tipo: "R"`) o estesa alle aree vicine (`tipo: "S"`) [V su codice di terzi] |
| Blocco delle date | la sola proposta, ~40 minuti, non rilasciabile | **tutti i posti restituiti** passano a `RISERVATO`; durata tra 8 e 15 minuti secondo le pagine regionali [R]; rilascio: [?] |
| Luogo di ogni data | testo da interpretare | campi separati: struttura, indirizzo, comune, provincia [V su codice di terzi] |

Le due chiamate note, dal progetto `VoidElle/tuscany-CUP-periodic-checker` (pubblicato il 10/11/2026; qui
serve come indizio da riverificare, non come codice da copiare):

- `GET …/getPrescrizioneElettronica?cf=…&nre=…&numbersTeam=…` → la ricetta con le prestazioni. Con dati
  sbagliati risponde comunque 200, con `response: null` e codice `999`. Gli identificativi cambiano a ogni
  chiamata: vanno riletti prima di ogni ricerca.
- `POST …/getListaDisponibilitaPost`, con l'intestazione `cf` obbligatoria → l'elenco degli appuntamenti con
  data, ora, stato e struttura. Nessun posto: elenco vuoto e codice `127`.

**Cosa cambia per l'app**

- Un campo in più per ricetta (cifre della tessera), chiesto solo per la Toscana, cifrato come gli altri.
- Il modulo del portale è molto più corto di quello piemontese: richieste JSON, niente stato di pagina.
- La zona si filtra sui campi comune e provincia della risposta, senza interpretare indirizzi.
- "Estendi area" diventa un parametro della ricerca.
- **Il blocco dei posti è il rischio principale.** In Piemonte un controllo tiene una data; in Toscana ne
  tiene una per struttura. Finché non si conosce la chiamata di rilascio (il portale deve averne una: è ciò
  che succede quando si torna indietro o scade il tempo), i controlli vanno fatti di rado (uno all'ora, come
  fa il progetto di terzi) e solo nell'area che l'utente ha scelto. Con il rilascio: cerca, confronta,
  rilascia subito ciò che non serve.
- Lo spostamento non è garantito per ogni prestazione. Dove manca, l'app si ferma all'avviso (vedi sezione
  6, punto G) finché non si è visto dal vivo che cosa fa il portale.
- Secondo lo stesso progetto il portale pubblica date nuove intorno a mezzanotte [R]: conviene un controllo
  poco dopo.
- Da dire agli utenti: nell'ASL Sud Est il percorso di tutela (la presa in carico quando non c'è posto nei
  tempi) non si attiva per le prenotazioni fatte online, solo da sportello o telefono [R]. Riguarda le
  ricette mai prenotate.

**Da scoprire dal vivo**: lettura degli appuntamenti esistenti, spostamento, disdetta, rilascio dei posti,
durata reale del blocco, numero di cifre della tessera, limiti di frequenza.

## 4. Calabria

**Portale**: `https://prenota.regione.calabria.it/home` ("CUP Calabria"), il SovraCUP regionale [R]. Non è
ancora del tutto unico: gli atti regionali di febbraio 2026 descrivono un modello misto, con i CUP aziendali
ancora operativi dietro il sistema centrale, e l'estensione completa è data per il 2027 [R]. Fornitore del
software: non trovato.

| | Piemonte | Calabria |
|---|---|---|
| Accesso anonimo | CF + NRE | CF + NRE, solo ricetta dematerializzata; mostra solo la prenotazione di quella ricetta [R]. Alcune fonti del 2026 parlano di SPID/CIE "per tutti i servizi": che l'accesso anonimo sia ancora attivo è [?] |
| Tecnologia | JSF/ICEfaces | ignota [?]; i percorsi `/home` e `/app/…` fanno pensare a un'applicazione a pagina singola con API |
| Captcha / anti-bot | disattivato | [?] |
| Lettura della prenotazione | "Recupera Prenotazioni" | "Recupera prenotazioni" [R] |
| Spostamento | sempre, in un passo | "modifica" come ricerca di una nuova disponibilità, con nuovo promemoria [R]; se sia un passo solo o disdetta + nuova prenotazione: [?] |
| Disdetta | sì | sì; termine tra 48 ore e 3 giorni secondo l'azienda [R] |
| Ricerca delle date | proposta + estensione | dall'ambito dell'assistito, allargabile; filtri per data e sede [R] |
| Blocco delle date | ~40 minuti | [?] |
| Senza identificarsi | niente | **"Prime disponibilità" e "Tempi di attesa" pubblici** [R] |

**Cosa cambia per l'app**

- Nessun campo in più, a quanto risulta.
- Tutto il modulo del portale è da scrivere partendo da zero: non c'è codice di terzi.
- La copertura non è uniforme: prima di promettere "Calabria" va verificato quali aziende rispondono
  davvero dal portale regionale (dichiarate: ASP di Catanzaro, Reggio, Cosenza, Crotone, Vibo; GOM di
  Reggio; AO di Cosenza; AOU Dulbecco) [R].
- "Prime disponibilità" permette una modalità in più, che in Piemonte non esiste: seguire una prestazione
  in una zona senza mandare codice fiscale e ricetta, e senza bloccare date.
- Esiste un'app ufficiale ("CUP Calabria", Android e iOS): è un secondo canale possibile, vedi sezione 6.

**Da scoprire dal vivo**: praticamente tutto ciò che è tecnico. La Calabria ha bisogno della ricognizione
più della Toscana.

## 5. Le domande da fare a ogni regione

Sono le stesse cinque caratteristiche del Piemonte, più due. Le risposte decidono quale strada della
sezione 6 si prende e diventano le "capacità" dichiarate dal modulo della regione (sezione 7).

| Domanda | Piemonte | Toscana | Calabria |
|---|---|---|---|
| 1. Con quali dati si entra senza login? | CF + NRE | CF + NRE + cifre tessera | CF + NRE [?] |
| 2. Captcha, OTP, anti-bot? | no | pare di no | [?] |
| 3. Si legge la prenotazione esistente? | sì | sì [?] | sì [?] |
| 4. Lo spostamento è un passo solo? | sì | dipende dalla prestazione | [?] |
| 5. La ricerca blocca date? Si rilasciano? | una, 40 min, no | tutte, 8-15 min, [?] | [?] |
| 6. Come si allarga la zona? | pulsante, fino a 4 volte | parametro | filtri [?] |
| 7. C'è qualcosa di pubblico senza identità? | no | tempi medi | prime disponibilità |

## 6. Se la tecnica del Piemonte non funziona: le alternative

In ordine di preferenza. Le prime tre non richiedono nulla all'utente oltre ai dati della ricetta.

**A. API JSON dirette.** Quando il portale è un'applicazione a pagina singola, le chiamate sono JSON senza
stato: più semplici e più stabili dei moduli JSF. È il caso della Toscana e probabilmente della Calabria.
Gli script della pagina contengono i nomi di tutte le chiamate, anche di quelle di spostamento e disdetta:
si possono leggere senza una ricetta.

**B. Solo avviso, e si conclude sul portale.** L'app trova la data e avvisa; l'utente sposta dal sito, con
"Copia" per codice fiscale e ricetta (c'è già). È il ripiego sempre disponibile quando le scritture non sono
note o non sono sicure, ed è il primo livello di ogni regione nuova.

**C. Monitor pubblico, senza identità.** Dove il portale mostra le prime disponibilità per prestazione e
zona (Calabria), si può avvisare senza mandare dati personali e senza bloccare date. Meno preciso: non sa
qual è la tua prenotazione né la tua priorità.

**D. La sessione dell'utente in una finestra dell'app.** Per le regioni che vogliono SPID/CIE, o dove un
controllo del browser (captcha, script di verifica) ferma le richieste dirette: l'utente entra nel portale
dentro una finestra dell'app (Tauri ha già un motore web) e l'app legge le date nella sua sessione. L'app
non vede e non conserva mai le credenziali SPID. Limite serio: le sessioni SPID durano poco, quindi il
controllo "anche a finestra chiusa, ogni 20 minuti" non regge; diventa "controlla finché sei collegato" o
"ricollegati una volta al giorno".

**E. Un browser guidato.** Come lo script originale (Playwright): regge qualsiasi portale ma pesa centinaia
di megabyte ed è lento. Utile in sviluppo per registrare le chiamate; nel prodotto solo come ultima scelta.

**F. Le API dell'app ufficiale.** Dove il canale senza SPID esiste solo nell'app della regione (Veneto) o
l'app è più completa del sito. Richiede di osservare il traffico dell'app, è più fragile e va valutata
rispetto alle condizioni d'uso. Da considerare solo se il sito non basta.

**G. Disdici e riprenota, dove lo spostamento non esiste.** È l'unico caso in cui si può restare senza
appuntamento. Si fa solo se il portale tiene per noi la data nuova mentre si disdice la vecchia, solo con
il "Sì" dell'utente che sa cosa rischia, mai in automatico, e solo dopo averlo visto funzionare dal vivo.
Fino ad allora, per quelle prestazioni vale B.

**H. I canali ufficiali.** L'app può dire all'utente ciò che la regione già offre e che pochi conoscono:
presa in carico e pre-lista in Toscana, percorsi di tutela in Calabria (oltre 120 giorni senza
disponibilità), call center. Nel lungo periodo si può chiedere alle regioni un accesso concordato: le
piattaforme hanno già servizi per terzi (la convenzione tra CUP Calabria e Poste lo dimostra), ma non sono
pubblici.

**Cosa non fare**: conservare credenziali SPID, usare servizi che risolvono captcha, aggirare protezioni
anti-bot. Se un portale mette una barriera esplicita, per quella regione si scende a B, C o D.

## 7. Come cambia il motore

L'idea: un **modulo per regione** dietro un'interfaccia comune, e tutto il resto condiviso. Ogni modulo
dichiara che cosa sa fare, e l'app mostra solo quello.

```
app/core/src/
  regioni/
    mod.rs        Regione (Piemonte, Toscana, Calabria), il tratto Portale, le Capacità
    piemonte/     portale.rs e analisi.rs di oggi, spostati senza cambiarli
    toscana/      client JSON
    calabria/
  geografia.rs    comuni, province e gruppi di comuni, per regione (oggi dentro zona.rs)
  zona.rs, distanza.rs, contatti.rs, errore.rs    generali
  operazioni.rs   la parte generale: fasi, conferma dell'utente, verifica a sessione nuova, esito incerto
```

- **`Regione`**, ricavata dal numero della ricetta: le prime tre cifre dell'NRE sono il codice della regione
  che l'ha emessa (010 Piemonte, 090 Toscana, 180 Calabria; codici ISTAT, verificati su fonte per 030, 042 e
  080). L'utente può cambiarla: una ricetta si può prenotare anche in un'altra regione.
- **Tratto `Portale`**: leggi la prenotazione, cerca le date, rilascia, prepara lo spostamento (fino al
  riepilogo), conferma, disdici, prenota una ricetta nuova. La sessione piemontese di oggi ne diventa la
  prima implementazione.
- **`Capacità`** per regione: dati richiesti oltre a CF e NRE, spostamento in un passo o no, quante date
  blocca una ricerca e per quanto, se si rilasciano, intervallo minimo tra due controlli, se c'è l'estensione
  dell'area, se c'è la prenotazione di una ricetta nuova, numero del call center.
- **`Slot`**: il campo `seleziona` (oggi il nome di un pulsante JSF) diventa un riferimento opaco che solo il
  modulo della regione sa usare. `Luogo` prende comune e provincia come campi, riempiti dal modulo: in
  Toscana arrivano già separati, in Piemonte li ricava il codice di oggi.
- **Geografia**: un file dei comuni per regione nello stesso formato di `comuni_piemonte.txt` (nome,
  provincia, coordinate; fonte ISTAT), le province per regione, e i gruppi come "Torino e prima cintura"
  come dato, non come scelta fissa dell'interfaccia.
- **Archivio dell'app**: ogni ricetta prende la regione e i dati in più (cifre della tessera). Le ricette
  già salvate, senza regione, valgono Piemonte: nessuna migrazione da fare a mano.
- **Sicurezza, uguale per tutti**: niente conferma se un controllo non torna; dopo la conferma ogni dubbio è
  "esito incerto", mai "non fatto"; verifica sempre con una sessione nuova; mai dati verso un indirizzo
  diverso da quello del portale. Queste regole oggi sono in `operazioni.rs` e restano lì.
- **Prove**: il finto portale di `prova.rs` resta lo strumento. Ogni regione ha le sue risposte registrate
  (rese anonime) e gli stessi scenari: data prima trovata, nessuna data, portale giù, risposta inattesa,
  esito incerto.
- **Impronte**: ciò che oggi il bot fa per le pagine del Piemonte (accorgersi che il portale è cambiato) va
  previsto per ogni regione. Per le API JSON è più semplice: si confronta la forma delle risposte.

Il bot Telegram resta sul Piemonte durante il pilota. È un secondo motore in un'altra lingua: portarlo su
più regioni raddoppia il lavoro di ogni modulo. Conviene decidere dopo il pilota se portarlo o se farlo
appoggiare al motore dell'app.

## 8. Il piano del pilota

Ogni fase ha un criterio d'uscita. Le fasi 2-4 valgono per regione e possono procedere sfalsate.

**Fase 0 — Ricognizione** (nessuna modifica al prodotto)
1. Aprire la rete dell'ambiente di lavoro ai due portali e leggerne pagine e script: elenco delle chiamate,
   campi richiesti, eventuali captcha, condizioni d'uso. Non serve una ricetta.
2. Uno strumento di registrazione generale, sul modello di `telegram-bot/dev/record_disdetta.py`: l'utente
   fa a mano nel browser lettura, ricerca, spostamento (fermandosi prima della conferma) e disdetta; lo
   strumento salva le chiamate con codice fiscale, ricetta e tessera mascherati.
3. Una registrazione per regione con una ricetta vera, meglio se già prenotata.
- *Uscita*: la tabella della sezione 5 senza [?] per Toscana e Calabria, e la decisione per ciascuna tra le
  strade della sezione 6.

**Fase 1 — Fondamenta** (il Piemonte non cambia)
- `Regione`, `Portale`, `Capacità`; il codice piemontese spostato nel suo modulo; geografia per regione;
  regione e dati in più nell'archivio; `--regione` nella riga di comando.
- *Uscita*: tutte le prove di oggi passano senza toccarle; l'app piemontese si comporta in modo identico.

**Fase 2 — Sola lettura**, prima dalla riga di comando, poi nell'app
- Leggere la prenotazione, cercare le date, filtrarle per zona e giorni, avvisare. In Toscana: rilascio dei
  posti e intervallo prudente. In Calabria: anche il monitor pubblico (strada C).
- *Uscita*: una settimana di controlli dal vivo su una ricetta vera, senza errori non spiegati e senza posti
  lasciati bloccati.

**Fase 3 — Sposta e disdici con il "Sì"**
- Le stesse garanzie del Piemonte: riepilogo confrontato con la data scelta, conferma, verifica a sessione
  nuova. Dove lo spostamento non è un passo solo, strada G o si resta alla fase 2.
- *Uscita*: almeno uno spostamento e una disdetta veri riusciti per regione, e i casi di errore provati sul
  finto portale.

**Fase 4 — Prenotazione automatica e ricetta mai prenotata**
- Solo dopo un periodo di uso della fase 3 senza esiti incerti.

**Fase 5 — Rilascio**
- Nome dell'app e degli installer, testi, pagina di download, canale degli aggiornamenti (vedi "Decisioni").
  La regione nuova esce segnata come "di prova", com'è stato per le ricette mai prenotate.

Rischi principali e come si tengono a bada:

| Rischio | Contromisura |
|---|---|
| I controlli tolgono posti agli altri (Toscana) | Rilascio subito dopo la ricerca; senza rilascio, controlli radi e solo nella zona scelta; intervallo minimo imposto dal modulo della regione |
| Un portale cambia e l'app sbaglia | Impronte per regione; a ogni risposta inattesa l'app si ferma e non conferma nulla |
| Si resta senza appuntamento (disdici + riprenota) | Mai in automatico; solo dove visto dal vivo; altrimenti solo avviso |
| Il portale chiude l'accesso anonimo o mette un captcha | Capacità per regione: l'app scende a "solo avviso" o alla finestra con la sessione dell'utente |
| Non si trova una ricetta vera per provare | La regione resta alla fase 2 o non esce |
| Il refactoring rompe il Piemonte | Fase 1 a comportamento identico, con le prove di oggi come rete |

## 9. Le altre regioni

Quadro provvisorio, tutto da fonti non aperte direttamente: prima di investire su una regione va rifatta la
fase 0. "Stessa tecnica" vuol dire accesso anonimo con i dati della ricetta.

| Gruppo | Regioni | Note |
|---|---|---|
| 1. Anonimo con CF + NRE | Sardegna, Marche, Friuli Venezia Giulia, Basilicata | Sardegna: ha "Sposta" ed esistono già tre bot di terzi; è però prevista una nuova gara per il CUP. Marche: le fonti ufficiali parlano di SPID, ma un progetto open source attivo usa CF + ricetta. Friuli: fonti datate. |
| 2. Anonimo con un dato in più | Lombardia, Liguria, Lazio | CF + ultime cifre della tessera + NRE, come la Toscana. Liguria ha "Cambia appuntamento". In Lombardia lo spostamento potrebbe chiedere il login. |
| 3. Anonimo con ostacoli | Puglia, Abruzzo, Veneto | Puglia: CUP aziendali dietro il portale. Abruzzo: registrazione con OTP, in passaggio a solo SPID. Veneto: senza SPID solo dall'app regionale, con un CUP per ULSS. |
| 4. Solo SPID/CIE | Emilia-Romagna, Campania, Sicilia, Umbria, Trento, Valle d'Aosta | Serve la strada D (la sessione dell'utente), con i suoi limiti. |
| Non classificabili | Bolzano, Molise | Fonti insufficienti. |

**Ordine proposto dopo il pilota**: Lombardia e Lazio (per numero di abitanti, e perché il campo in più
sarà già fatto per la Toscana), Sardegna e Marche (accesso più semplice, codice di terzi da confrontare),
Liguria, Friuli Venezia Giulia, Basilicata; poi Puglia e Veneto; per ultime le regioni solo SPID, tutte
insieme, perché condividono la stessa soluzione.

Sul piano nazionale non c'è una scorciatoia: la Piattaforma nazionale liste di attesa (Agenas) riceve dati
dalle regioni per il monitoraggio e non offre prenotazioni; il Fascicolo sanitario 2.0 rimanda ai portali
regionali dietro SPID; l'app IO porta promemoria, non disponibilità [R]. Non risultano servizi, pubblici o
commerciali, che avvisino quando si libera una data prima.

## 10. Decisioni e cose che servono

1. **Rete dell'ambiente di lavoro**: aggiungere `prenota.sanita.toscana.it` e `prenota.regione.calabria.it`
   (e `cup.isan.csi.it`, per confronto) agli indirizzi ammessi. Sblocca il primo punto della fase 0.
2. **Ricette vere**: chi può fornire una ricetta toscana e una calabrese per le registrazioni? Senza, si
   arriva alla lettura degli script ma non alle prove.
3. **App o anche bot?** Proposta: solo l'app durante il pilota.
4. **Nome e distribuzione.** L'app si chiama "CUP Piemonte", e il canale degli aggiornamenti e la chiave di
   firma in `tauri.conf.json` sono quelli del repository `ilmondovero`, da cui questo è un fork. Va deciso
   se l'estensione torna lì o se questo fork pubblica versioni proprie (nome, chiave e canale nuovi). Non
   blocca le fasi 0-3.
5. **Codice di terzi per la Toscana**: usato solo come indizio. Se si volesse riusarne parti va verificata
   la licenza.

## 11. Fonti principali

Toscana
- https://prenota.sanita.toscana.it/
- https://www.regione.toscana.it/-/cup-online
- https://www.regione.toscana.it/-/prenotare-visite-ed-esami
- https://www.regione.toscana.it/-/privacy-prenotazioni-sanita
- https://github.com/VoidElle/tuscany-CUP-periodic-checker
- https://github.com/mercu-lore/CUP_Tracker
- https://askanews.it/2025/02/26/sanita-in-toscana-185-milioni-di-operazioni-cup-2-0-nel-2024/
- https://www.uslcentro.toscana.it/diagnosi-e-cura/la-prenotazione-di-visite-e-esami/prenotare-un-esame-o-visita/cosa-serve-per-prenotare
- https://www.uslsudest.toscana.it/come-fare-per/prenotare-visite-o-esami/istruzioni-per-la-prenotazione

Calabria
- https://prenota.regione.calabria.it/home
- https://www.regione.calabria.it/wp-content/uploads/2024/12/Descrizione-Video-Portale-Web-Cup-Calabria-1.pdf
- https://www.regione.calabria.it/wp-content/uploads/2024/01/allegato_linee_guida-(1).pdf
- https://www.regione.calabria.it/wp-content/uploads/2026/02/Atto_numero_1548_del_06-02-2026.pdf
- https://www.regione.calabria.it/wp-content/uploads/2025/07/CONVENZIONE-POLIS.pdf
- https://www.asprc.it/servizi/prenotazioni-cup-e-pagamento-ticket/prenotazioni-cup/
- https://play.google.com/store/apps/details?id=cup.regione.calabria.it

Altre regioni e quadro nazionale
- https://www.regione.lombardia.it/sanita/prenotazioni-e-tempi-d-attesa/prenotazione-online-visite-esami
- https://www.fascicolo-sanitario.it/fse/servizionline
- https://github.com/denilnicolosi/cup_marche_notifier
- https://www.innova.puglia.it/documents/10180/595001/SAUSSC+-+Specifiche+di+integrazione+servizi+Portale+Salute+-+CUP+aziendali+v1.11.pdf/2d28b20e-1305-44e4-96f5-c0622bf2e522
- https://www.quotidianosanita.it/governo-e-parlamento/liste-d-attesa-in-gu-il-decreto-con-le-nuove-linee-guida-per-la-piattaforma-nazionale-un-passo-avanti-verso-il-governo-centralizzato/
- https://docs.pagopa.it/i-modelli-dei-servizi/salute/in-arrivo

Struttura del numero di ricetta
- https://salute.regione.emilia-romagna.it/siseps/sanita/assistenza-farmaceutica/files/Disciplinare_tecnico_Gestione_Cannabis_AFO_AFT_web.pdf
