//! Quanto e' lontana una data: la distanza tra il comune da cui parte l'utente e quello della sede, e la scelta tra la
//! data piu' vicina nel tempo e quella piu' vicina nello spazio.
//!
//! Come si calcola, e perche' cosi':
//! - **Dove sono le cose.** Il portale scrive di ogni sede solo l'indirizzo, e da li' si legge il comune
//!   (`zona::comune`). Le coordinate sono quelle del centro dei comuni del Piemonte, gia' dentro l'app
//!   (`core/data/comuni_piemonte.txt`): una tabella in memoria, letta una volta, con ricerca per nome in tempo
//!   costante. Niente servizi di geocodifica in rete: il comune dell'utente e i luoghi delle visite non escono dal
//!   computer, e il calcolo funziona anche se quei servizi non rispondono.
//! - **La distanza.** Quella in linea d'aria sulla sfera terrestre (formula dell'emisenoverso, "haversine"): qualche
//!   operazione in virgola mobile per coppia di punti, esatta a ogni distanza. Si arrotonda al km: i punti sono i
//!   centri dei comuni, una precisione maggiore sarebbe finta. Due sedi nello stesso comune sono a 0 km dall'utente
//!   e fra loro conta la data.
//! - **L'ordine.** Le date arrivano dal portale gia' dalla piu' vicina nel tempo. La piu' vicina nello spazio e' il
//!   minimo per km in un solo passaggio (O(n), una distanza per data), a parita' di km la prima nel tempo: la prima
//!   data disponibile nella sede piu' vicina. Per mettere in fila tutte le date basta un ordinamento stabile per km
//!   (O(n log n), una distanza per data). Le date di cui non si legge il comune vanno in fondo.

use crate::errore::Errore;
use crate::zona;

/// Il raggio medio della Terra, in km.
const RAGGIO_TERRA: f64 = 6371.0088;

/// Cosa preferisce l'utente quando ci sono piu' date buone: la prima (vicinanza nel tempo) o la sede piu' vicina
/// (vicinanza nello spazio).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Criterio {
    /// La prima data buona, dove che sia (nella zona scelta).
    #[default]
    Data,
    /// La prima data buona nella sede piu' vicina al comune di partenza.
    Distanza,
}

impl Criterio {
    /// "data" (o vuoto, come le ricette salvate prima che ci fosse la scelta) o "distanza".
    pub fn da_scelta(nome: &str) -> Result<Criterio, Errore> {
        match nome.trim() {
            "" | "data" => Ok(Criterio::Data),
            "distanza" => Ok(Criterio::Distanza),
            altro => Err(Errore::DatiNonValidi(format!("Preferenza sconosciuta: {altro}"))),
        }
    }

    pub fn nome(self) -> &'static str {
        match self {
            Criterio::Data => "data",
            Criterio::Distanza => "distanza",
        }
    }
}

/// Un punto sulla Terra: latitudine e longitudine in radianti.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Punto {
    lat: f64,
    lon: f64,
}

impl Punto {
    /// Da latitudine e longitudine in gradi.
    pub fn new(lat: f64, lon: f64) -> Self {
        Punto { lat: lat.to_radians(), lon: lon.to_radians() }
    }

    /// Il centro di un comune del Piemonte ("Torino", "MONDOVI'"); None se non e' un comune del Piemonte.
    pub fn di(comune: &str) -> Option<Self> {
        zona::centro(comune).map(|(lat, lon)| Punto::new(lat, lon))
    }

    /// I km in linea d'aria fino a un altro punto (formula dell'emisenoverso).
    pub fn km(&self, altro: &Punto) -> f64 {
        let mezzo_lat = (altro.lat - self.lat) / 2.0;
        let mezzo_lon = (altro.lon - self.lon) / 2.0;
        let a = mezzo_lat.sin().powi(2) + self.lat.cos() * altro.lat.cos() * mezzo_lon.sin().powi(2);
        2.0 * RAGGIO_TERRA * a.sqrt().min(1.0).asin()
    }

    /// I km interi fino al centro di un comune; None se il comune non si trova (vuoto, fuori dal Piemonte, scritto
    /// male dal portale).
    pub fn km_da(&self, comune: &str) -> Option<u32> {
        Punto::di(comune).map(|p| self.km(&p).round() as u32)
    }
}

/// "stesso comune", "12 km": la distanza come si legge nell'app e nelle notifiche.
pub fn km_breve(km: u32) -> String {
    if km == 0 {
        "stesso comune".into()
    } else {
        format!("{km} km")
    }
}

/// La data nella sede piu' vicina e, fra le date li', la prima: il minimo per km in un solo passaggio. Le date vanno
/// date dalla piu' vicina nel tempo (come le da' il portale): a parita' di km vince la prima. Quelle di cui non si sa
/// la distanza non contano; None se non se ne sa nessuna.
pub fn la_piu_vicina<T>(date: impl IntoIterator<Item = T>, km: impl Fn(&T) -> Option<u32>) -> Option<(T, u32)> {
    let mut scelta: Option<(T, u32)> = None;
    for d in date {
        if let Some(k) = km(&d) {
            if scelta.as_ref().is_none_or(|(_, migliore)| k < *migliore) {
                scelta = Some((d, k));
            }
        }
    }
    scelta
}

/// Mette in fila le date (date dalla piu' vicina nel tempo) per il criterio: per data restano come sono; per distanza
/// dalla sede piu' vicina, a parita' di km dalla prima nel tempo (l'ordinamento e' stabile e calcola una distanza per
/// data), quelle di cui non si sa la distanza in fondo.
pub fn ordina<T>(date: &mut [T], criterio: Criterio, km: impl Fn(&T) -> Option<u32>) {
    if criterio == Criterio::Distanza {
        date.sort_by_cached_key(|d| km(d).unwrap_or(u32::MAX));
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn distanze_in_linea_d_aria_tra_i_comuni() {
        let torino = Punto::di("Torino").unwrap();
        // in linea d'aria: Torino-Asti ~ 46 km, Torino-Cuneo ~ 76 km, Torino-Novara ~ 85 km
        let km = |c: &str| torino.km_da(c).unwrap();
        assert!((43..=49).contains(&km("Asti")), "{}", km("Asti"));
        assert!((73..=79).contains(&km("Cuneo")), "{}", km("Cuneo"));
        assert!((82..=88).contains(&km("Novara")), "{}", km("Novara"));
        assert!(km("Moncalieri") <= 10);
        // lo stesso comune, scritto come il portale ("MONDOVI'") o come l'utente ("mondovi")
        assert_eq!(km("TORINO"), 0);
        assert_eq!(Punto::di("mondovi").unwrap().km_da("MONDOVI'"), Some(0));
        // simmetrica
        let asti = Punto::di("Asti").unwrap();
        assert_eq!(asti.km_da("Torino"), torino.km_da("Asti"));
        // fuori dal Piemonte o vuoto: non si sa
        assert_eq!(torino.km_da("Parigi"), None);
        assert_eq!(torino.km_da(""), None);
        assert_eq!(Punto::di("Milano"), None);
        // un grado di latitudine e' ~111 km
        assert_eq!(Punto::new(45.0, 7.0).km(&Punto::new(46.0, 7.0)).round(), 111.0);
        assert_eq!(Punto::new(45.0, 7.0).km(&Punto::new(45.0, 7.0)), 0.0);
        assert_eq!((km_breve(0), km_breve(12)), ("stesso comune".into(), "12 km".into()));
    }

    #[test]
    fn la_prima_data_nella_sede_piu_vicina() {
        // (giorno, km): dalla piu' vicina nel tempo, come le da' il portale
        let date = [(1, Some(40)), (2, Some(12)), (3, None), (4, Some(12)), (5, Some(0)), (6, Some(0))];
        let km = |d: &&(u32, Option<u32>)| d.1;
        // la piu' vicina nello spazio e, li', la prima nel tempo
        assert_eq!(la_piu_vicina(date.iter(), km).map(|(d, k)| (d.0, k)), Some((5, 0)));
        // a parita' di km la prima nel tempo
        assert_eq!(la_piu_vicina(date[..5].iter().filter(|d| d.0 != 5), km).map(|(d, _)| d.0), Some(2));
        // senza distanze note: nessuna
        assert!(la_piu_vicina([(1, None::<u32>)].iter(), |d| d.1).is_none());
        assert!(la_piu_vicina(Vec::<(u32, Option<u32>)>::new(), |d| d.1).is_none());
        // tutte in fila: per distanza (stabile, quelle senza distanza in fondo) o come sono
        let mut fila = date.to_vec();
        ordina(&mut fila, Criterio::Distanza, |d| d.1);
        assert_eq!(fila.iter().map(|d| d.0).collect::<Vec<_>>(), [5, 6, 2, 4, 1, 3]);
        let mut fila = date.to_vec();
        ordina(&mut fila, Criterio::Data, |d| d.1);
        assert_eq!(fila, date);
    }

    #[test]
    fn criterio_scelto_dall_utente() {
        assert_eq!(Criterio::da_scelta("").unwrap(), Criterio::Data); // le ricette salvate prima della scelta
        assert_eq!(Criterio::da_scelta("data").unwrap(), Criterio::Data);
        assert_eq!(Criterio::da_scelta(" distanza ").unwrap(), Criterio::Distanza);
        assert!(Criterio::da_scelta("vicino").is_err());
        assert_eq!(Criterio::da_scelta(Criterio::Distanza.nome()).unwrap(), Criterio::Distanza);
        assert_eq!(Criterio::default(), Criterio::Data);
    }
}
