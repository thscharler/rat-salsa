use lipsum::lipsum;
use rat_text::text_area::TextAreaState;
use rat_text::TextRange;
use ropey::{Rope, RopeBuilder};
use std::ops::Range;

pub fn add_range_styles(state: &mut TextAreaState, styles: Vec<(TextRange, usize)>) {
    for (range, style) in styles {
        _ = state.add_range_style(range, style);
    }
}

#[allow(unused)]
pub fn sample_scott_0() -> (Rope, Vec<(TextRange, usize)>) {
    let rope = Rope::from_str(SCOTT_0);
    let mut styles = Vec::new();

    styles.push((TextRange::new((0, 0), (13, 0)), 0));
    styles.push((TextRange::new((0, 1), (13, 1)), 0));
    styles.push((TextRange::new((4, 3), (17, 3)), 0));
    styles.push((TextRange::new((31, 44), (44, 44)), 0));

    // overlapping styles
    styles.push((TextRange::new((30, 7), (42, 7)), 0));
    styles.push((TextRange::new((37, 7), (41, 7)), 1));

    styles.push((TextRange::new((44, 7), (63, 7)), 0));
    styles.push((TextRange::new((58, 7), (62, 7)), 1));

    styles.push((TextRange::new((65, 7), (6, 8)), 0));
    styles.push((TextRange::new((1, 8), (5, 8)), 0));

    styles.push((TextRange::new((8, 8), (24, 8)), 0));
    styles.push((TextRange::new((19, 8), (23, 8)), 0));

    styles.push((TextRange::new((26, 8), (48, 8)), 0));
    styles.push((TextRange::new((43, 8), (47, 8)), 0));

    styles.push((TextRange::new((53, 8), (73, 8)), 0));
    styles.push((TextRange::new((68, 8), (72, 8)), 0));

    (rope, styles)
}

#[allow(unused)]
pub fn sample_scott_1() -> (Rope, Vec<(TextRange, usize)>) {
    (Rope::from_str(SCOTT_1), Vec::new())
}

#[allow(unused)]
pub fn sample_bosworth_1() -> (Rope, Vec<(TextRange, usize)>) {
    (Rope::from_str(BOSWORTH), Vec::new())
}

#[allow(unused)]
pub fn sample_emoji() -> (Rope, Vec<(TextRange, usize)>) {
    (
        Rope::from_str("short text\nw🤷‍♂️x\nw🤷‍♀️x\nw🤦‍♂️x\nw❤️x\nw🤦‍♀️x\nw💕x\nw🙍🏿‍♀️x\n"),
        Vec::new(),
    )
}

#[allow(unused)]
pub fn sample_tabs() -> (Rope, Vec<(TextRange, usize)>) {
    (
        Rope::from_str("\t\ttabs\n\t\t\t\ttabs\n\tt\tt\tt\n"),
        Vec::new(),
    )
}

#[allow(unused)]
pub fn sample_lorem_ipsum() -> (Rope, Vec<(TextRange, usize)>) {
    let styles = Vec::new();
    let mut buf = RopeBuilder::new();

    let words = lipsum(2500);
    buf.append(words.as_str());

    let rope = buf.finish();

    (rope, styles)
}

#[allow(unused)]
pub fn sample_lorem_rustum() -> (Rope, Vec<(Range<usize>, usize)>) {
    let l = lorem_rustum::LoremRustum::new(1_000_000);

    let mut styles = Vec::new();

    let mut buf = RopeBuilder::new();
    let mut pos = 0;
    let mut width = 0;
    for p in l.body {
        buf.append(p);
        buf.append(" ");
        width += p.len() + 1;

        if p == "macro" {
            styles.push((pos..pos + p.len(), 0));
        } else if p == "assert!" {
            styles.push((pos..pos + p.len(), 1));
        } else if p == "<'a>" {
            styles.push((pos..pos + p.len(), 2));
        } else if p == "await" {
            styles.push((pos..pos + p.len(), 3));
        }

        pos += p.len() + 1;

        if width > 66 {
            buf.append("\n");
            width = 0;
            pos += 1;
        }
    }
    let buf = buf.finish();

    (buf, styles)
}

#[allow(unused)]
pub fn sample_pattern_0() -> (Rope, Vec<(TextRange, usize)>) {
    (Rope::from_str(PATTERN_0), Vec::new())
}

#[allow(unused)]
pub fn sample_long() -> (Rope, Vec<(TextRange, usize)>) {
    let mut buf = String::new();
    let pat = ["1", "2", "3", "4", " ", "6", "7", "8", "9", " "];

    for i in 0..500 {
        use std::fmt::Write;

        _ = write!(buf, "{:04} ", i);
        for j in 0..128000 {
            buf.push_str(pat[j % 10]);
        }
        buf.push_str("\n");
    }

    (Rope::from(buf), Vec::new())
}

#[allow(unused)]
pub fn sample_medium() -> (Rope, Vec<(TextRange, usize)>) {
    let mut buf = String::new();
    let pat = ["1", "2", "3", "4", " ", "6", "7", "8", "9", " "];

    for i in 0..500 {
        use std::fmt::Write;

        _ = write!(buf, "{:04} ", i);
        for j in 0..16384 {
            buf.push_str(pat[j % 10]);
        }
        buf.push_str("\n");
    }

    (Rope::from(buf), Vec::new())
}

#[allow(unused)]
pub fn sample_short() -> (Rope, Vec<(TextRange, usize)>) {
    let mut buf = String::new();
    let pat = ["1", "2", "3", "4", " ", "6", "7", "8", "9", " "];

    for i in 0..500 {
        use std::fmt::Write;

        _ = write!(buf, "{:04} ", i);
        for j in 0..1024 {
            buf.push_str(pat[j % 10]);
        }
        buf.push_str("\n");
    }

    (Rope::from(buf), Vec::new())
}

#[allow(unused)]
static PATTERN_0: &str = "aaaa 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 \n\
    bbbb 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 \n\
    cccc 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 \n\
    dddd 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 \n\
    eeee 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 \n\
    ffff 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 \n\
    gggg 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 \n\
    hhhh 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 \n\
    iiii 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 \n\
    jjjj 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 \n\
    kkkk 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 \n\
    llll 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 \n\
    mmmm 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 \n\
    nnnn 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 \n\
    oooo 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 \n\
    pppp 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 \n\
    qqqq 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 \n\
    rrrr 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 \n\
    ssss 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 \n\
    tttt 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 \n\
    uuuu 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 \n\
    vvvv 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 \n\
    wwww 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 \n\
    xxxx 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 \n\
    yyyy 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 \n\
    zzzz 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 1234 6789 \n\
    ";

#[allow(unused)]
static SCOTT_0: &str = "Ridley Scott
Ridley Scott (2015)

Sir Ridley Scott GBE[1] (* 30. November 1937 in South Shields, England) ist ein
britischer Filmregisseur und Filmproduzent. Er gilt heute als einer der
renommiertesten und einflussreichsten Regisseure und hat die Erzählweisen
mehrerer Filmgenres geprägt. Er wurde nie mit einem Oscar ausgezeichnet.
Seine bekanntesten Filme sind Alien (1979), Blade Runner (1982), Thelma & Louise
(1991), Gladiator (2000), Black Hawk Down (2001) und Der Marsianer (2015).

Scott ist Eigentümer der 1995 gegründeten Filmproduktionsfirma Scott Free Productions.

Inhaltsverzeichnis

    1 Leben
    2 Werk
    3 Filmografie (Auswahl)
    4 Auszeichnungen (Auswahl)
    5 Literatur
    6 Weblinks
    7 Einzelnachweise

Leben

Scott wurde als Sohn eines Berufssoldaten geboren. Sein Vater, den er selten
zu sehen bekam, diente bei den Royal Engineers (Kampfunterstützungstruppen der
britischen Armee) in hoher Position als Ingenieur und Transportkontrolleur.
Nach Aufenthalten in Cumbria, Wales und Deutschland (dort zwischen 1947 und
1952 in Hamburg) ließ sich die Familie in Stockton-on-Tees im Norden Englands
nieder (die industriell geprägte Landschaft inspirierte später Szenen in Blade
Runner). Zum Ende seiner Kindheit und Jugend hatte er eigenen Angaben zufolge
wegen der vielen Umzüge 10 Schulen besucht.[2]

Scott erlernte 1954 bis 1958 Grafikdesign und Malerei am West Hartlepool College
of Art und erlangte das Diplom mit Auszeichnung. Er studierte daraufhin
Grafikdesign (M.A., 1960 bis 1962) am Royal College of Art in London, wo
David Hockney einer seiner Mitstudenten war. Er schloss 1963 mit Auszeichnung
ab. Scott erhielt ein einjähriges Reisestipendium in die USA und wurde bei Time Life
beschäftigt, wo er mit den Dokumentaristen Richard Leacock und D. A. Pennebaker
arbeitete. Nach seiner Rückkehr nahm er 1965 eine Lehrstelle bei der BBC als
Szenenbildner an. Diese Position führte ihn zur Mitarbeit an beliebten
Fernsehproduktionen wie der Polizei-Serie Z-Cars oder der Science-Fiction-Serie
Out of the Unknown. Nach kurzer Zeit wurde er ins Trainingsprogramm für Regisseure
aufgenommen und inszenierte einige Episoden selbst.

1968 verließ Scott die BBC, um Ridley Scott Associates (RSA) zu gründen. An dem
Projekt arbeiteten neben seinem Bruder Tony Regisseure wie Alan Parker, Hugh Hudson
und Hugh Johnson mit. RSA wurde zu einem der erfolgreichsten Werbefilm-Häuser in
Europa, in dessen Auftrag Scott für über 2000 Werbespots verantwortlich zeichnet;
viele davon wurden auf den Festspielen von Cannes und Venedig ausgezeichnet.

Scott gilt in der Branche als ökonomischer Regisseur, da er in der Regel mit einem
Drittel der Drehtage seiner Kollegen auskommt. Eigenen Worten zufolge verdankt
er dies seiner Vergangenheit als Werbe- und Videospotregisseur sowie der Tatsache,
dass er manche Szenen mit bis zu 15 Kameras gleichzeitig drehe.[3] Seit dem Jahr 2000,
als sie in Gladiator eine Nebenrolle spielte, ist Scott mit der costa-ricanischen
Schauspielerin Giannina Facio, Tochter des Diplomaten und Politikers Gonzalo Facio
(1918–2018), liiert. Im Juni 2015 heiratete das Paar.[4]

Im Jahr 2003 wurde Scott von der britischen Königin aufgrund seiner Verdienste um
die Kunst zum Ritter geschlagen, am 8. Mai 2024 ernannte Thronfolger Prinz William
ihn zum Knight Grand Cross of the Order of the British Empire. Scott ist damit
Träger des höchsten britischen Verdienstordens.

Sein jüngerer Bruder ist der Regisseur und Filmproduzent Tony Scott, der sich 2012
das Leben nahm. Seine Söhne Luke und Jake und seine Tochter Jordan sind ebenfalls
im Filmgeschäft tätig.

Scott lebt in Los Angeles, besitzt aber seit etwa Anfang der 90er Jahre ein Haus
in Südfrankreich.[2]

Werk

Scotts Markenzeichen ist ein ausgeprägt ästhetischer und malerischer visueller Stil,
der sich durch seine jahrelange Erfahrung als Production Designer und Regisseur
von Werbespots entwickelt hat. Zusammen mit seinem Bruder Tony betrieb er ab
1968 die Produktionsfirma für Werbefilme Ridley Scott Associates (RSA).

Scotts erster Themenfilm Die Duellisten (1977) war zwar kommerziell kein großer
Erfolg, fand aber bei der Kritik genug Beachtung, um Scott die Realisierung des
Science-Fiction-Films Alien – Das unheimliche Wesen aus einer fremden Welt (1979)
zu ermöglichen. Sein nächster Film Blade Runner (1982), basierend auf dem Roman
Träumen Androiden von elektrischen Schafen? von Philip K. Dick, spielt in einem
düster-futuristischen Los Angeles. Das Werk war visuell derart beeindruckend, dass
es für eine ganze Generation Cyberpunk-Literatur, -Musik und -Kunst als Inspiration
diente. In der Folge drehte Scott Legende (1985), Der Mann im Hintergrund (1987)
und Black Rain (1989), die alle nicht an die Bedeutung und den Erfolg der vorigen
Werke anknüpfen konnten. Legende setzte sich jedoch im Lauf der Zeit als Fantasy-Kultfilm
durch und wurde 2002 mit einem restaurierten Director’s Cut ergänzt.

Die von der Kritik stetig vorgebrachte Beschuldigung, visuellen Stil vor Inhalt und
Charakterzeichnung zu stellen, wurde mit Thelma & Louise (1991) entkräftet. Neben
guten Kritiken erhielt Scott seine erste Oscar-Nominierung für die beste Regie.
Danach folgten mit dem Kolumbus-Film 1492 – Die Eroberung des Paradieses (1992),
White Squall – Reißende Strömung (1996) und Die Akte Jane (1997) erneut Filme, die
künstlerisch und kommerziell durchfielen. Insbesondere der Militärfilm Die Akte Jane,
in dem Demi Moore eine Frau spielt, die als erste Mitglied bei den Navy Seals
werden will, wurde wegen einer nach Ansicht vieler Kritiker undifferenzierten
Pro-Militär-Haltung angegriffen. Mit Gladiator feierte Scott 2000 ein triumphales
Comeback. Der Film war beim Publikum sehr erfolgreich und gewann neben dem Oscar
für den besten Film im Jahr 2000 auch den Golden Globe 2001. Die Regie-Leistung
wurde ebenfalls nominiert, den Preis erhielt Scott jedoch nicht. Eine weitere
Oscar-Nominierung erhielt er für den kontroversen Kriegsfilm Black Hawk Down
(2001), der einen verunglückten US-amerikanischen Militäreinsatz in Somalia
thematisiert und in eindrucksvolle Bilder umsetzt. Black Hawk Down prägte die
neuere Action-Darstellung und verhalf der dokumentaristischen Kameraführung zum
Durchbruch in der Filmkunst.

Scott übernahm die Regie bei dem Film Hannibal (2001), der Fortsetzung zu Das
Schweigen der Lämmer (1991). 2005/2006 folgte in zwei Versionen der Film Königreich
der Himmel. 2006 erschien Ein gutes Jahr nach dem Roman Ein guter Jahrgang seines
Landsmannes Peter Mayle. Er handelt von einem Bankmanager, der von seinem Onkel
ein Weingut in der Provence erbt und daraufhin beschließt, sein Leben umzukrempeln.
Die Hauptrolle spielt der neuseeländische Schauspieler Russell Crowe. Gemeinsam
mit seinem Bruder Tony produzierte Scott für den amerikanischen Kabelsender TNT
die Miniserie The Company – Im Auftrag der CIA, die im August 2007 ausgestrahlt
wurde. The Company erzählt die Geschichte dreier Yale-Absolventen, die in der
Nachkriegszeit auf Seiten der CIA bzw. des KGB in den Kalten Krieg verwickelt
werden. In den Hauptrollen sind u. a. Chris O’Donnell, Michael Keaton und Alfred
Molina zu sehen.

Im Oktober 2008 bestätigte Scott, dass er 25 Jahre warten musste, bis die Rechte an
dem Buch Der Ewige Krieg von Joe Haldeman für eine Verfilmung zur Verfügung standen.
[5] Scott plane, dieses Buch in 3D zu verfilmen.[6]

Für den US-Fernsehsender CBS produzierte Scott seit 2009 die Serie Good Wife.
Die Ausstrahlung begann in den USA im September 2009, in Deutschland bei ProSieben
Ende März 2010. Auch hier arbeitete er mit seinem Bruder Tony zusammen. Mit der
2009 abgedrehten Produktion Robin Hood legte Scott erneut einen Historienfilm
vor. Mit seinem 22. Spielfilm, realisiert nach einem Drehbuch von Brian Helgeland
mit Russell Crowe in der Titelrolle, wurden am 12. Mai 2010 die 63. Filmfestspiele
von Cannes eröffnet.[7]

Scott arbeitete 2009 an der ersten Verfilmung von Aldous Huxleys Roman Schöne neue
Welt für das Kino. Der Film sollte von ihm und Leonardo DiCaprio produziert werden,
Drehbuchautor sollte Farhad Safinia sein. Scott sollte voraussichtlich auch Regie
führen, der Film wurde jedoch bis heute nicht realisiert.[8] Der Film Prometheus
war ursprünglich als Prequel zu Scotts erstem großen Erfolg Alien geplant. Das
Drehbuch stammt von Jon Spaihts; Damon Lindelof überarbeitete das Drehbuch für
20th Century Fox. In den USA erfolgte der Kinostart am 8. Juni 2012. 2017
folgte die Fortsetzung Alien: Covenant. Im selben Jahr verfilmte Scott mit
Alles Geld der Welt den Entführungsfall um John Paul Getty III. Im Zuge des
Skandals um Kevin Spacey, der ab Ende Oktober 2017 mit Vorwürfen der sexuellen
Belästigung konfrontiert wurde, entschloss sich das Filmteam und Sony Pictures,
alle Szenen mit Spacey aus dem Film zu schneiden. Scott musste diese Szenen
kurzfristig mit Christopher Plummer nachdrehen. ";

static SCOTT_1: &str = "Ridley Scott
Ridley Scott (2015)

Sir Ridley Scott GBE[1] (* 30. November 1937 in South Shields, England) ist ein britischer Filmregisseur und Filmproduzent. Er gilt heute als einer der renommiertesten und einflussreichsten Regisseure und hat die Erzählweisen mehrerer Filmgenres geprägt. Er wurde nie mit einem Oscar ausgezeichnet. Seine bekanntesten Filme sind Alien (1979), Blade Runner (1982), Thelma & Louise (1991), Gladiator (2000), Black Hawk Down (2001) und Der Marsianer (2015).

Scott ist Eigentümer der 1995 gegründeten Filmproduktionsfirma Scott Free Productions.

Inhaltsverzeichnis

    1 Leben
    2 Werk
    3 Filmografie (Auswahl)
    4 Auszeichnungen (Auswahl)
    5 Literatur
    6 Weblinks
    7 Einzelnachweise

Leben

Scott wurde als Sohn eines Berufs\u{200B}soldaten geboren. Sein Vater, den er selten zu sehen bekam, diente bei den Royal Engineers (Kampf\u{00AD}unterstützungs\u{00AD}truppen der britischen Armee) in hoher Position als Ingenieur und Transport\u{00AD}kontrolleur. Nach Aufenthalten in Cumbria, Wales und Deutschland (dort zwischen 1947 und 1952 in Hamburg) ließ sich die Familie in Stockton-on-Tees im Norden Englands nieder (die industriell geprägte Landschaft inspirierte später Szenen in Blade Runner). Zum Ende seiner Kindheit und Jugend hatte er eigenen Angaben zufolge wegen der vielen Umzüge 10 Schulen besucht.[2]

Scott erlernte 1954 bis 1958 Grafikdesign und Malerei am West Hartlepool College of Art und erlangte das Diplom mit Auszeichnung. Er studierte daraufhin Grafikdesign (M.A., 1960 bis 1962) am Royal College of Art in London, wo David Hockney einer seiner Mitstudenten war. Er schloss 1963 mit Auszeichnung ab. Scott erhielt ein einjähriges Reisestipendium in die USA und wurde bei Time Life beschäftigt, wo er mit den Dokumentaristen Richard Leacock und D. A. Pennebaker arbeitete. Nach seiner Rückkehr nahm er 1965 eine Lehrstelle bei der BBC als Szenenbildner an. Diese Position führte ihn zur Mitarbeit an beliebten Fernsehproduktionen wie der Polizei-Serie Z-Cars oder der Science-Fiction-Serie Out of the Unknown. Nach kurzer Zeit wurde er ins Trainingsprogramm für Regisseure aufgenommen und inszenierte einige Episoden selbst.

1968 verließ Scott die BBC, um Ridley Scott Associates (RSA) zu gründen. An dem Projekt arbeiteten neben seinem Bruder Tony Regisseure wie Alan Parker, Hugh Hudson und Hugh Johnson mit. RSA wurde zu einem der erfolgreichsten Werbefilm-Häuser in Europa, in dessen Auftrag Scott für über 2000 Werbespots verantwortlich zeichnet; viele davon wurden auf den Festspielen von Cannes und Venedig ausgezeichnet.

Scott gilt in der Branche als ökonomischer Regisseur, da er in der Regel mit einem Drittel der Drehtage seiner Kollegen auskommt. Eigenen Worten zufolge verdankt er dies seiner Vergangenheit als Werbe- und Videospot\u{00AD}regisseur sowie der Tatsache, dass er manche Szenen mit bis zu 15 Kameras gleichzeitig drehe.[3] Seit dem Jahr 2000, als sie in Gladiator eine Nebenrolle spielte, ist Scott mit der costa-ricanischen Schauspielerin Giannina Facio, Tochter des Diplomaten und Politikers Gonzalo Facio (1918–2018), liiert. Im Juni 2015 heiratete das Paar.[4]

Im Jahr 2003 wurde Scott von der britischen Königin aufgrund seiner Verdienste um die Kunst zum Ritter geschlagen, am 8. Mai 2024 ernannte Thronfolger Prinz William ihn zum Knight Grand Cross of the Order of the British Empire. Scott ist damit Träger des höchsten britischen Verdienstordens.

Sein jüngerer Bruder ist der Regisseur und Filmproduzent Tony Scott, der sich 2012 das Leben nahm. Seine Söhne Luke und Jake und seine Tochter Jordan sind ebenfalls im Filmgeschäft tätig.

Scott lebt in Los Angeles, besitzt aber seit etwa Anfang der 90er Jahre ein Haus in Südfrankreich.[2]

Werk

Scotts Markenzeichen ist ein ausgeprägt ästhetischer und malerischer visueller Stil, der sich durch seine jahrelange Erfahrung als Production Designer und Regisseur von Werbespots entwickelt hat. Zusammen mit seinem Bruder Tony betrieb er ab 1968 die Produktionsfirma für Werbefilme Ridley Scott Associates (RSA).

Scotts erster Themenfilm Die Duellisten (1977) war zwar kommerziell kein großer Erfolg, fand aber bei der Kritik genug Beachtung, um Scott die Realisierung des Science-Fiction-Films Alien – Das unheimliche Wesen aus einer fremden Welt (1979) zu ermöglichen. Sein nächster Film Blade Runner (1982), basierend auf dem Roman Träumen Androiden von elektrischen Schafen? von Philip K. Dick, spielt in einem düster-futuristischen Los Angeles. Das Werk war visuell derart beeindruckend, dass es für eine ganze Generation Cyberpunk-Literatur, -Musik und -Kunst als Inspiration diente. In der Folge drehte Scott Legende (1985), Der Mann im Hintergrund (1987) und Black Rain (1989), die alle nicht an die Bedeutung und den Erfolg der vorigen Werke anknüpfen konnten. Legende setzte sich jedoch im Lauf der Zeit als Fantasy-Kultfilm durch und wurde 2002 mit einem restaurierten Director’s Cut ergänzt.

Die von der Kritik stetig vorgebrachte Beschuldigung, visuellen Stil vor Inhalt und Charakter\u{00AD}zeichnung zu stellen, wurde mit Thelma & Louise (1991) entkräftet. Neben guten Kritiken erhielt Scott seine erste Oscar-Nominierung für die beste Regie. Danach folgten mit dem Kolumbus-Film 1492 – Die Eroberung des Paradieses (1992), White Squall – Reißende Strömung (1996) und Die Akte Jane (1997) erneut Filme, die künstlerisch und kommerziell durchfielen. Insbesondere der Militärfilm Die Akte Jane, in dem Demi Moore eine Frau spielt, die als erste Mitglied bei den Navy Seals werden will, wurde wegen einer nach Ansicht vieler Kritiker undifferenzierten Pro-Militär-Haltung angegriffen. Mit Gladiator feierte Scott 2000 ein triumphales Comeback. Der Film war beim Publikum sehr erfolgreich und gewann neben dem Oscar für den besten Film im Jahr 2000 auch den Golden Globe 2001. Die Regie-Leistung wurde ebenfalls nominiert, den Preis erhielt Scott jedoch nicht. Eine weitere Oscar-Nominierung erhielt er für den kontroversen Kriegsfilm Black Hawk Down (2001), der einen verunglückten US-amerikanischen Militäreinsatz in Somalia thematisiert und in eindrucksvolle Bilder umsetzt. Black Hawk Down prägte die neuere Action-Darstellung und verhalf der dokumentaristischen Kameraführung zum Durchbruch in der Filmkunst.

Scott übernahm die Regie bei dem Film Hannibal (2001), der Fortsetzung zu Das Schweigen der Lämmer (1991). 2005/2006 folgte in zwei Versionen der Film Königreich der Himmel. 2006 erschien Ein gutes Jahr nach dem Roman Ein guter Jahrgang seines Landsmannes Peter Mayle. Er handelt von einem Bankmanager, der von seinem Onkel ein Weingut in der Provence erbt und daraufhin beschließt, sein Leben umzukrempeln. Die Hauptrolle spielt der neuseeländische Schauspieler Russell Crowe. Gemeinsam mit seinem Bruder Tony produzierte Scott für den amerikanischen Kabelsender TNT die Miniserie The Company – Im Auftrag der CIA, die im August 2007 ausgestrahlt wurde. The Company erzählt die Geschichte dreier Yale-Absolventen, die in der Nachkriegszeit auf Seiten der CIA bzw. des KGB in den Kalten Krieg verwickelt werden. In den Hauptrollen sind u. a. Chris O’Donnell, Michael Keaton und Alfred Molina zu sehen.

Im Oktober 2008 bestätigte Scott, dass er 25 Jahre warten musste, bis die Rechte an dem Buch Der Ewige Krieg von Joe Haldeman für eine Verfilmung zur Verfügung standen. [5] Scott plane, dieses Buch in 3D zu verfilmen.[6]
 
Für den US-Fernsehsender CBS produzierte Scott seit 2009 die Serie Good Wife. Die Ausstrahlung begann in den USA im September 2009, in Deutschland bei ProSieben Ende März 2010. Auch hier arbeitete er mit seinem Bruder Tony zusammen. Mit der 2009 abgedrehten Produktion Robin Hood legte Scott erneut einen Historienfilm vor. Mit seinem 22. Spielfilm, realisiert nach einem Drehbuch von Brian Helgeland mit Russell Crowe in der Titelrolle, wurden am 12. Mai 2010 die 63. Filmfestspiele von Cannes eröffnet.[7]

Scott arbeitete 2009 an der ersten Verfilmung von Aldous Huxleys Roman Schöne neue Welt für das Kino. Der Film sollte von ihm und Leonardo DiCaprio produziert werden, Drehbuchautor sollte Farhad Safinia sein. Scott sollte voraussichtlich auch Regie führen, der Film wurde jedoch bis heute nicht realisiert.[8] Der Film Prometheus war ursprünglich als Prequel zu Scotts erstem großen Erfolg Alien geplant. Das Drehbuch stammt von Jon Spaihts; Damon Lindelof überarbeitete das Drehbuch für 20th Century Fox. In den USA erfolgte der Kinostart am 8. Juni 2012. 2017 folgte die Fortsetzung Alien: Covenant. Im selben Jahr verfilmte Scott mit Alles Geld der Welt den Entführungsfall um John Paul Getty III. Im Zuge des Skandals um Kevin Spacey, der ab Ende Oktober 2017 mit Vorwürfen der sexuellen Belästigung konfrontiert wurde, entschloss sich das Filmteam und Sony Pictures, alle Szenen mit Spacey aus dem Film zu schneiden. Scott musste diese Szenen kurzfristig mit Christopher Plummer nachdrehen. ";

static BOSWORTH: &str = "Battle of Bosworth Field

The Battle of Bosworth or Bosworth Field (/ˈbɒzwərθ/ BOZ-wərth) was the last significant battle of the Wars of the Roses, the civil war between the houses of Lancaster and York that extended across England in the latter half of the 15th century. Fought on 22 August 1485, the battle was won by an alliance of Lancastrians and disaffected Yorkists. Their leader Henry Tudor, Earl of Richmond, became the first English monarch of the Tudor dynasty by his victory and subsequent marriage to a Yorkist princess. His opponent Richard III, the last king of the House of York, was killed during the battle, the last English monarch to fall in battle. Historians consider Bosworth Field to mark the end of the Plantagenet dynasty, making it one of the defining moments of English history.

Richard's reign began in 1483 when he ascended the throne after his twelve-year-old nephew, Edward V, was declared illegitimate, likely at Richard’s instigation. The boy and his younger brother Richard soon disappeared, and their fate remains a mystery. Across the English Channel Henry Tudor, a descendant of the greatly diminished House of Lancaster, seized on Richard's difficulties and laid claim to the throne. Henry's first attempt to invade England in 1483 foundered in a storm, but his second arrived unopposed on 7 August 1485 on the south-west coast of Wales. Marching inland, Henry gathered support as he made for London. Richard hurriedly mustered his troops and intercepted Henry's army near Ambion Hill, south of the town of Market Bosworth in Leicestershire. Lord Stanley and Sir William Stanley also brought a force to the battlefield, but held back while they decided which side it would be most advantageous to support, initially lending only four knights to Henry's cause; these were: Sir Robert Tunstall, Sir John Savage (nephew of Lord Stanley), Sir Hugh Persall and Sir Humphrey Stanley.[3] Sir John Savage was placed in command of the left flank of Henry's army.

Richard divided his army, which outnumbered Henry's, into three groups (or \"battles\"). One was assigned to the Duke of Norfolk and another to the Earl of Northumberland. Henry kept most of his force together and placed it under the command of the experienced Earl of Oxford. Richard's vanguard, commanded by Norfolk, attacked but struggled against Oxford's men, and some of Norfolk's troops fled the field. Northumberland took no action when signalled to assist his king, so Richard gambled everything on a charge across the battlefield to kill Henry and end the fight. Seeing the king's knights separated from his army, the Stanleys intervened; Sir William led his men to Henry's aid, surrounding and killing Richard. After the battle, Henry was crowned king.

Henry hired chroniclers to portray his reign favourably; the Battle of Bosworth Field was popularised to represent his Tudor dynasty as the start of a new age, marking the end of the Middle Ages for England. From the 15th to the 18th centuries the battle was glamourised as a victory of good over evil, and features as the climax of William Shakespeare's play Richard III. The exact site of the battle is disputed because of the lack of conclusive data, and memorials have been erected at different locations. The Bosworth Battlefield Heritage Centre was built in 1974, on a site that has since been challenged by several scholars and historians. In October 2009, a team of researchers who had performed geological surveys and archaeological digs in the area since 2003 suggested a location two miles (3.2 km) south-west of Ambion Hill.

Background

During the 15th century civil war raged across England as the Houses of York and Lancaster fought each other for the English throne. In 1471 the Yorkists defeated their rivals in the battles of Barnet and Tewkesbury. The Lancastrian King Henry VI and his only son, Edward of Westminster, died in the aftermath of the Battle of Tewkesbury. Their deaths left the House of Lancaster with no direct claimants to the throne. The Yorkist king, Edward IV, was in complete control of England.[4] He attainted those who refused to submit to his rule, such as Jasper Tudor and his nephew Henry, naming them traitors and confiscating their lands. The Tudors tried to flee to France but strong winds forced them to land in Brittany, which was a semi-independent duchy, where they were taken into the custody of Duke Francis II.[5] Henry's mother, Lady Margaret Beaufort, was a great-granddaughter of John of Gaunt, uncle of King Richard II and father of King Henry IV.[6] The Beauforts were originally bastards, but Richard II legitimised them through an Act of Parliament, a decision quickly modified by a royal decree of Henry IV ordering that their descendants were not eligible to inherit the throne.[7] Henry Tudor, the only remaining Lancastrian noble with a trace of the royal bloodline, had a weak claim to the throne,[4] and Edward regarded him as \"a nobody\".[8] The Duke of Brittany, however, viewed Henry as a valuable tool to bargain for England's aid in conflicts with France, and kept the Tudors under his protection.[8]

Edward IV died 12 years after Tewkesbury in April 1483.[9] His 12-year-old elder son succeeded him as King Edward V; the younger son, nine-year-old Richard of Shrewsbury, Duke of York, was next in line to the throne. Edward V was too young to rule and a Royal Council was established to rule the country until the king's coming of age. Some among the council were worried when it became apparent that the relatives of Edward V's mother, Elizabeth Woodville, were plotting to use their control of the young king to dominate the council.[10] Having offended many in their quest for wealth and power, the Woodville family was not popular.[11] To frustrate the Woodvilles' ambitions, Lord Hastings and other members of the council turned to the new king's uncle—Richard, Duke of Gloucester, brother of Edward IV. The courtiers urged Gloucester to assume the role of Protector quickly, as had been previously requested by his now dead brother.[12] On 29 April Gloucester, accompanied by a contingent of guards and Henry Stafford, 2nd Duke of Buckingham, took Edward V into custody and arrested several prominent members of the Woodville family.[13] After bringing the young king to London, Gloucester had the Queen's brother Anthony Woodville, 2nd Earl Rivers, and her son by her first marriage Richard Grey executed, without trial, on charges of treason.[14]

On 13 June, Gloucester accused Hastings of plotting with the Woodvilles and had him beheaded.[15] Nine days later the Three Estates of the Realm, an informal Parliament declared the marriage between Edward IV and Elizabeth illegal, rendering their children illegitimate and disqualifying them from the throne.[16] With his brother's children out of the way, he was next in the line of succession and was proclaimed King Richard III on 26 June.[17] The timing and extrajudicial nature of the deeds done to obtain the throne for Richard won him no popularity, and rumours that spoke ill of the new king spread throughout England.[18] After they were declared bastards, the two princes were confined in the Tower of London and never seen in public again.[19]

In October 1483, a conspiracy emerged to displace him from the throne. The rebels were mostly loyalists to Edward IV, who saw Richard as a usurper.[20] Their plans were coordinated by a Lancastrian, Henry's mother Lady Margaret, who was promoting her son as a candidate for the throne. The highest-ranking conspirator was Buckingham. No chronicles tell of the duke's motive in joining the plot, although historian Charles Ross proposes that Buckingham was trying to distance himself from a king who was becoming increasingly unpopular with the people.[21] Michael Jones and Malcolm Underwood suggest that Margaret deceived Buckingham into thinking the rebels supported him to be king.[22]

The plan was to stage uprisings within a short time in southern and western England, overwhelming Richard's forces. Buckingham would support the rebels by invading from Wales, while Henry came in by sea.[23] Bad timing and weather wrecked the plot. An uprising in Kent started 10 days prematurely, alerting Richard to muster the royal army and take steps to put down the insurrections. Richard's spies informed him of Buckingham's activities, and the king's men captured and destroyed the bridges across the River Severn. When Buckingham and his army reached the river, they found it swollen and impossible to cross because of a violent storm that broke on 15 October.[24] Buckingham was trapped and had no safe place to retreat; his Welsh enemies seized his home castle after he had set forth with his army. The duke abandoned his plans and fled to Wem, where he was betrayed by his servant and arrested by Richard's men.[25] On 2 November he was executed.[26] Henry had attempted a landing on 10 October (or 19 October), but his fleet was scattered by a storm. He reached the coast of England (at either Plymouth or Poole) and a group of soldiers hailed him to come ashore. They were, in fact, Richard's men, prepared to capture Henry once he set foot on English soil. Henry was not deceived and returned to Brittany, abandoning the invasion.[27] Without Buckingham or Henry, the rebellion was easily crushed by Richard.[26]

The survivors of the failed uprisings fled to Brittany, where they openly supported Henry's claim to the throne.[28] At Christmas, Henry Tudor swore an oath in Rennes Cathedral to marry Edward IV's daughter, Elizabeth of York, to unite the warring houses of York and Lancaster.[29] Henry's rising prominence made him a great threat to Richard, and the Yorkist king made several overtures to the Duke of Brittany to surrender the young Lancastrian. Francis refused, holding out for the possibility of better terms from Richard.[30] In mid-1484 Francis was incapacitated by illness and while recuperating, his treasurer Pierre Landais took over the reins of government. Landais reached an agreement with Richard to send back Henry and his uncle in exchange for military and financial aid. John Morton, a bishop of Flanders, learned of the scheme and warned the Tudors, who fled to France.[31] The French court allowed them to stay; the Tudors were useful pawns to ensure that Richard's England did not interfere with French plans to annex Brittany.[32] On 16 March 1485 Richard's queen, Anne Neville, died,[33] and rumours spread across the country that she was murdered to pave the way for Richard to marry his niece, Elizabeth. Later findings though, showed that Richard had entered into negotiations to marry Joanna of Portugal and to marry off Elizabeth to Manuel, Duke of Beja.[34] The gossip must have upset Henry across the English Channel.[35] The loss of Elizabeth's hand in marriage could unravel the alliance between Henry's supporters who were Lancastrians and those who were loyalists to Edward IV.[36] Anxious to secure his bride, Henry recruited mercenaries formerly in French service to supplement his following of exiles and set sail from France on 1 August.[37]

Factions

By the 15th century, English chivalric ideas of selfless service to the king had been corrupted.[38] Armed forces were raised mostly through musters in individual estates; every able-bodied man had to respond to his lord's call to arms, and each noble had authority over his militia. Although a king could raise personal militia from his lands, he could muster a large army only through the support of his nobles. Richard, like his predecessors, had to win over these men by granting gifts and maintaining cordial relationships.[39] Powerful nobles could demand greater incentives to remain on the liege's side or else they might turn against him.[40] Three groups, each with its own agenda, stood on Bosworth Field: Richard III and his Yorkist army; his challenger, Henry Tudor, who championed the Lancastrian cause; and the fence-sitting Stanleys.[41]

Yorkist

Small and slender, Richard III did not have the robust physique associated with many of his Plantagenet predecessors.[42] However, he enjoyed very rough sports and activities that were considered manly.[43] His performances on the battlefield impressed his brother greatly, and he became Edward's right-hand man.[44] During the 1480s Richard defended the northern borders of England. In 1482, Edward charged him to lead an army into Scotland with the aim of replacing King James III with the Duke of Albany.[45] Richard's army broke through the Scottish defences and occupied the capital, Edinburgh, but Albany decided to give up his claim to the throne in return for the post of Lieutenant General of Scotland. As well as obtaining a guarantee that the Scottish government would concede territories and diplomatic benefits to the English crown, Richard's campaign retook the town of Berwick-upon-Tweed, which the Scots had conquered in 1460.[46] Edward was not satisfied by these gains,[47] which, according to Ross, could have been greater if Richard had been resolute enough to capitalise on the situation while in control of Edinburgh.[48] In her analysis of Richard's character, Christine Carpenter sees him as a soldier who was more used to taking orders than giving them.[49] However, he was not averse to displaying his militaristic streak; on ascending the throne he made known his desire to lead a crusade against \"not only the Turks, but all [his] foes\".[43]

Richard's most loyal subject was John Howard, 1st Duke of Norfolk.[50] The duke had served Richard's brother for many years and had been one of Edward IV's closer confidants.[51] He was a military veteran, having fought in the Battle of Towton in 1461 and served as Hastings' deputy at Calais in 1471.[52] Ross speculates that he bore a grudge against Edward for depriving him of a fortune. Norfolk was due to inherit a share of the wealthy Mowbray estate on the death of eight-year-old Anne de Mowbray, the last of her family. However, Edward convinced Parliament to circumvent the law of inheritance and transfer the estate to his younger son, who was married to Anne. Consequently, Howard supported Richard III in deposing Edward's sons, for which he received the dukedom of Norfolk and his original share of the Mowbray estate.[53]

Henry Percy, 4th Earl of Northumberland, also supported Richard's ascension to the throne of England. The Percys were loyal Lancastrians, but Edward IV eventually won the earl's allegiance. Northumberland had been captured and imprisoned by the Yorkists in 1461, losing his titles and estates; however, Edward released him eight years later and restored his earldom.[54] From that time Northumberland served the Yorkist crown, helping to defend northern England and maintain its peace.[55] Initially the earl had issues with Richard III as Edward groomed his brother to be the leading power of the north. Northumberland was mollified when he was promised he would be the Warden of the East March, a position that was formerly hereditary for the Percys.[56] He served under Richard during the 1482 invasion of Scotland, and the allure of being in a position to dominate the north of England if Richard went south to assume the crown was his likely motivation for supporting Richard's bid for kingship.[57] However, after becoming king, Richard began moulding his nephew, John de la Pole, 1st Earl of Lincoln, to manage the north, passing over Northumberland for the position. According to Carpenter, although the earl was amply compensated, he despaired of any possibility of advancement under Richard.[58]

Lancastrians

Henry Tudor was unfamiliar with the arts of war and was a stranger to the land he was trying to conquer. He spent the first fourteen years of his life in Wales and the next fourteen in Brittany and France.[59] Slender but strong and decisive, Henry lacked a penchant for battle and was not much of a warrior; chroniclers such as Polydore Vergil and ambassadors like Pedro de Ayala found him more interested in commerce and finance.[60] Having not fought in any battles,[61] Henry recruited several experienced veterans to command his armies.[62] John de Vere, 13th Earl of Oxford, was Henry's principal military commander.[63] He was adept in the arts of war. At the Battle of Barnet, he commanded the Lancastrian right wing and routed the division opposing him. However, as a result of confusion over identities, Oxford's group came under friendly fire from the Lancastrian main force and retreated from the field. The earl fled abroad and continued his fight against the Yorkists, raiding shipping and eventually capturing the island fort of St Michael's Mount in 1473. He surrendered after receiving no aid or reinforcement, but in 1484 escaped from prison and joined Henry's court in France, bringing along his erstwhile gaoler Sir James Blount.[64] Oxford's presence raised morale in Henry's camp and troubled Richard III.[65]

Stanleys

In the early stages of the Wars of the Roses, the Stanleys of Cheshire had been predominantly Lancastrians.[66] Sir William Stanley, however, was a staunch Yorkist supporter, fighting in the Battle of Blore Heath in 1459 and helping Hastings to put down uprisings against Edward IV in 1471.[67] When Richard took the crown, Sir William showed no inclination to turn against the new king, refraining from joining Buckingham's rebellion, for which he was amply rewarded.[68] Sir William's elder brother, Thomas Stanley, 2nd Baron Stanley, was not as steadfast. By 1485, he had served three kings, namely Henry VI, Edward IV and Richard III. Lord Stanley's skilled political manoeuvrings—vacillating between opposing sides until it was clear who would be the winner—gained him high positions;[69] he was Henry's chamberlain and Edward's steward.[70] His non-committal stance, until the crucial point of a battle, earned him the loyalty of his men, who felt he would not needlessly send them to their deaths.[65]

Lord Stanley's relations with the king's brother, the eventual Richard III, were not cordial. The two had conflicts that erupted into violence around March 1470.[71] Furthermore, having taken Lady Margaret as his second wife in June 1472,[72] Stanley was Henry Tudor's stepfather, a relationship which did nothing to win him Richard's favour. Despite these differences, Stanley did not join Buckingham's revolt in 1483.[68] When Richard executed those conspirators who had been unable to flee England,[26] he spared Lady Margaret. However, he declared her titles forfeit and transferred her estates to Stanley's name, to be held in trust for the Yorkist crown. Richard's act of mercy was calculated to reconcile him with Stanley,[22] but it may have been to no avail—Carpenter has identified a further cause of friction in Richard's intention to reopen an old land dispute that involved Thomas Stanley and the Harrington family.[73] Edward IV had ruled the case in favour of Stanley in 1473,[74] but Richard planned to overturn his brother's ruling and give the wealthy estate to the Harringtons.[73] Immediately before the Battle of Bosworth, being wary of Stanley, Richard took his son, Lord Strange, as hostage to discourage him from joining Henry.[75]

Crossing the English Channel and through Wales

Henry's initial force consisted of the English and Welsh exiles who had gathered around Henry, combined with a contingent of mercenaries put at his disposal by Charles VIII of France. The history of Scottish author John Major (published in 1521) claims that Charles had granted Henry 5,000 men, of whom 1,000 were Scots, headed by Sir Alexander Bruce. No mention of Scottish soldiers was made by subsequent English historians.[76]

Henry's crossing of the English Channel in 1485 was without incident. Thirty ships sailed from Harfleur on 1 August and, with fair winds behind them, landed in his native Wales, at Mill Bay (near Dale) on the north side of Milford Haven on 7 August, easily capturing nearby Dale Castle.[77] Henry received a muted response from the local population. No joyous welcome awaited him on shore, and at first few individual Welshmen joined his army as it marched inland.[78] Historian Geoffrey Elton suggests only Henry's ardent supporters felt pride over his Welsh blood.[79] His arrival had been hailed by contemporary Welsh bards such as Dafydd Ddu and Gruffydd ap Dafydd as the true prince and \"the youth of Brittany defeating the Saxons\" in order to bring their country back to glory.[80][81] When Henry moved to Haverfordwest, the county town of Pembrokeshire, Richard's lieutenant in South Wales, Sir Walter Herbert, failed to move against Henry, and two of his officers, Richard Griffith and Evan Morgan, deserted to Henry with their men.[82]

The most important defector to Henry in this early stage of the campaign was probably Rhys ap Thomas, who was the leading figure in West Wales.[82] Richard had appointed Rhys Lieutenant in West Wales for his refusal to join Buckingham's rebellion, asking that he surrender his son Gruffydd ap Rhys ap Thomas as surety, although by some accounts Rhys had managed to evade this condition. However, Henry successfully courted Rhys, offering the lieutenancy of all Wales in exchange for his fealty. Henry marched via Aberystwyth while Rhys followed a more southerly route, recruiting a force of Welshmen en route, variously estimated at 500 or 2,000 men, to swell Henry's army when they reunited at Cefn Digoll, Welshpool.[83] By 15 or 16 August, Henry and his men had crossed the English border, making for the town of Shrewsbury.[84]

Since 22 June Richard had been aware of Henry's impending invasion, and had ordered his lords to maintain a high level of readiness.[85] News of Henry's landing reached Richard on 11 August, but it took three to four days for his messengers to notify his lords of their king's mobilisation. On 16 August, the Yorkist army started to gather; Norfolk set off for Leicester, the assembly point, that night. The city of York, a historical stronghold of Richard's family, asked the king for instructions, and receiving a reply three days later sent 80 men to join the king. Simultaneously Northumberland, whose northern territory was the most distant from the capital, had gathered his men and ridden to Leicester.[86]

Although London was his goal,[87] Henry did not move directly towards the city. After resting in Shrewsbury, his forces went eastwards and picked up Sir Gilbert Talbot and other English allies, including deserters from Richard's forces. Although its size had increased substantially since the landing, Henry's army was still considerably outnumbered by Richard's forces. Henry's pace through Staffordshire was slow, delaying the confrontation with Richard so that he could gather more recruits to his cause.[88] Henry had been communicating on friendly terms with the Stanleys for some time before setting foot in England,[36] and the Stanleys had mobilised their forces on hearing of Henry's landing. They ranged themselves ahead of Henry's march through the English countryside,[89] meeting twice in secret with Henry as he moved through Staffordshire.[90] At the second of these, at Atherstone in Warwickshire, they conferred \"in what sort to arraign battle with King Richard, whom they heard to be not far off\".[91] On 21 August, the Stanleys were making camp on the slopes of a hill north of Dadlington, while Henry encamped his army at White Moors to the north-west of their camp.[92]

On 20 August, Richard rode from Nottingham to Leicester,[93] joining Norfolk. He spent the night at the Blue Boar inn (demolished 1836).[93] Northumberland arrived the following day. The royal army proceeded westwards to intercept Henry's march on London. Passing Sutton Cheney, Richard moved his army towards Ambion Hill—which he thought would be of tactical value—and made camp on it.[92] Richard's sleep was not peaceful and, according to the Croyland Chronicle, in the morning his face was \"more livid and ghastly than usual\".[94]

Engagement

The Yorkist army, variously estimated at between 7,500 and 12,000 men, deployed on the hilltop[95][96] along the ridgeline from west to east. Norfolk's force (or \"battle\" in the parlance of the time) of spearmen stood on the right flank, protecting the cannon and about 1,200 archers. Richard's group, comprising 3,000 infantry, formed the centre. Northumberland's men guarded the left flank; he had approximately 4,000 men, many of them mounted.[97] Standing on the hilltop, Richard had a wide, unobstructed view of the area. He could see the Stanleys and their 4,000–6,000 men holding positions on and around Dadlington Hill, while to the south-west was Henry's army.[98]

Henry's force has been variously estimated at between 5,000 and 8,000 men, his original landing force of exiles and mercenaries having been augmented by the recruits gathered in Wales and the English border counties (in the latter area probably mustered chiefly by the Talbot interest), and by deserters from Richard's army. Historian John Mackie believes that 1,800 French mercenaries, led by Philibert de Chandée, formed the core of Henry's army.[99] John Mair, writing thirty-five years after the battle, claimed that this force contained a significant Scottish component,[100] and this claim is accepted by some modern writers,[101] but Mackie argues that the French would not have released their elite Scottish knights and archers, and concludes that there were probably few Scottish troops in the army, although he accepts the presence of captains like Bernard Stewart, Lord of Aubigny.[99][100]

In their interpretations of the vague mentions of the battle in the old text, historians placed areas near the foot of Ambion Hill as likely regions where the two armies clashed, and thought up possible scenarios of the engagement.[102][103][104] In their recreations of the battle, Henry started by moving his army towards Ambion Hill where Richard and his men stood. As Henry's army advanced past the marsh at the south-western foot of the hill, Richard sent a message to Stanley, threatening to execute his son, Lord Strange, if Stanley did not join the attack on Henry immediately. Stanley replied that he had other sons. Incensed, Richard gave the order to behead Strange but his officers temporised, saying that battle was imminent, and it would be more convenient to carry out the execution afterwards.[105] Henry had also sent messengers to Stanley asking him to declare his allegiance. The reply was evasive—the Stanleys would \"naturally\" come, after Henry had given orders to his army and arranged them for battle. Henry had no choice but to confront Richard's forces alone.[41]

Well aware of his own military inexperience, Henry handed command of his army to Oxford and retired to the rear with his bodyguards. Oxford, seeing the vast line of Richard's army strung along the ridgeline, decided to keep his men together instead of splitting them into the traditional three battles: vanguard, centre, and rearguard. He ordered the troops to stray no further than 10 feet (3.0 m) from their banners, fearing that they would become enveloped. Individual groups clumped together, forming a single large mass flanked by horsemen on the wings.[106]

The Lancastrians were harassed by Richard's cannon as they manoeuvred around the marsh, seeking firmer ground.[107] Once Oxford and his men were clear of the marsh, Norfolk's battle and several contingents of Richard's group, under the command of Sir Robert Brackenbury, started to advance. Hails of arrows showered both sides as they closed. Oxford's men proved the steadier in the ensuing hand-to-hand combat; they held their ground and several of Norfolk's men fled the field.[108] Norfolk lost one of his senior officers, Walter Devereux, in this early clash.[109]

Recognising that his force was at a disadvantage, Richard signalled for Northumberland to assist but Northumberland's group showed no signs of movement. Historians, such as Horrox and Pugh, believe Northumberland chose not to aid his king for personal reasons.[110] Ross doubts the aspersions cast on Northumberland's loyalty, suggesting instead that Ambion Hill's narrow ridge hindered him from joining the battle. The earl would have had to either go through his allies or execute a wide flanking move—near impossible to perform given the standard of drill at the time—to engage Oxford's men.[111]

At this juncture Richard saw Henry at some distance behind his main force.[112] Seeing this, Richard decided to end the fight quickly by killing the enemy commander. He led a charge of mounted men around the melee and tore into Henry's group; several accounts state that Richard's force numbered 800–1000 knights, but Ross says it was more likely that Richard was accompanied only by his household men and closest friends.[113] Richard killed Henry's standard-bearer Sir William Brandon in the initial charge and unhorsed burly John Cheyne, Edward IV's former standard-bearer,[114] with a blow to the head from his broken lance.[1] French mercenaries in Henry's retinue related how the attack had caught them off guard and that Henry sought protection by dismounting and concealing himself among them to present less of a target. Henry made no attempt to engage in combat himself.[115]

Oxford had left a small reserve of pike-equipped men with Henry. They slowed the pace of Richard's mounted charge, and bought Tudor some critical time.[116] The remainder of Henry's bodyguards surrounded their master, and succeeded in keeping him away from the Yorkist king. Meanwhile, seeing Richard embroiled with Henry's men and separated from his main force, William Stanley made his move and rode to the aid of Henry. Now outnumbered, Richard's group was surrounded and gradually pressed back.[1] Richard's force was driven several hundred yards away from Tudor, near to the edge of a marsh, into which the king's horse toppled. Richard, now unhorsed, gathered himself and rallied his dwindling followers, supposedly refusing to retreat: \"God forbid that I retreat one step. I will either win the battle as a king, or die as one.\"[117] In the fighting Richard's banner man—Sir Percival Thirlwall—lost his legs, but held the Yorkist banner aloft until he was killed. It is likely that James Harrington also died in the charge.[118][119] The king's trusted advisor Richard Ratcliffe was also slain.[120]

Polydore Vergil, Henry Tudor's official historian, recorded that \"King Richard, alone, was killed fighting manfully in the thickest press of his enemies\".[121] Richard had come within a sword's length of Henry Tudor before being surrounded by William Stanley's men and killed. The Burgundian chronicler Jean Molinet says that a Welshman struck the death-blow with a halberd while Richard's horse was stuck in the marshy ground.[122] It was said that the blows were so violent that the king's helmet was driven into his skull.[123] The contemporary Welsh poet Guto'r Glyn implies the leading Welsh Lancastrian Rhys ap Thomas, or one of his men, killed the king, writing that he \"Lladd y baedd, eilliodd ei ben\" (\"Killed the boar, shaved his head\").[122][124] Analysis of King Richard's skeletal remains found 11 wounds, nine of them to the head; a blade consistent with a halberd had sliced off part of the rear of Richard's skull, suggesting he had lost his helmet.[125]

Richard's forces disintegrated as news of his death spread. Northumberland and his men fled north on seeing the king's fate,[1] and Norfolk was killed by the knight Sir John Savage in single combat according to the Ballad of Lady Bessy.[126]

After the battle

Although he claimed[127] fourth-generation maternal Lancastrian descendancy, Henry seized the crown by right of conquest. After the battle, Richard's circlet is said to have been found and brought to Henry, who was proclaimed king at the top of Crown Hill, near the village of Stoke Golding. According to Vergil, Henry's official historian, Lord Stanley found the circlet. Historians Stanley Chrimes and Sydney Anglo dismiss the legend of the circlet's finding in a hawthorn bush; none of the contemporary sources reported such an event.[1] Ross, however, does not ignore the legend. He argues that the hawthorn bush would not be part of Henry's coat of arms if it did not have a strong relationship to his ascendance.[128] Baldwin points out that a hawthorn bush motif was already used by the House of Lancaster, and Henry merely added the crown.[129]

In Vergil's chronicle, 100 of Henry's men, compared to 1,000 of Richard's, died in this battle—a ratio Chrimes believes to be an exaggeration.[1] The bodies of the fallen were brought to St James Church at Dadlington for burial.[130] However, Henry denied any immediate rest for Richard; instead the last Yorkist king's corpse was stripped naked and strapped across a horse. His body was brought to Leicester and openly exhibited to prove that he was dead. Early accounts suggest that this was in the major Lancastrian collegiate foundation, the Church of the Annunciation of Our Lady of the Newarke.[131] After two days, the corpse was interred in a plain tomb,[132] within the church of the Greyfriars.[133] The church was demolished following the friary's dissolution in 1538, and the location of Richard's tomb was long uncertain.[134]

On 12 September 2012, archaeologists announced the discovery of a buried skeleton with spinal abnormalities and head injuries under a car park in Leicester, and their suspicions that it was Richard III.[135] On 4 February 2013, it was announced that DNA testing had convinced Leicester University scientists and researchers \"beyond reasonable doubt\" that the remains were those of King Richard.[136] On 26 March 2015, these remains were ceremonially buried in Leicester Cathedral.[137] Richard's tomb was unveiled on the following day.[138]

Henry dismissed the mercenaries in his force, retaining only a small core of local soldiers to form a \"Yeomen of his Garde\",[139] and proceeded to establish his rule of England. Parliament reversed his attainder and recorded Richard's kingship as illegal, although the Yorkist king's reign remained officially in the annals of England history. The proclamation of Edward IV's children as illegitimate was also reversed, restoring Elizabeth's status to a royal princess.[140] The marriage of Elizabeth, the heiress to the House of York, to Henry, the master of the House of Lancaster, marked the end of the feud between the two houses and the start of the Tudor dynasty. The royal matrimony, however, was delayed until Henry was crowned king and had established his claim on the throne firmly enough to preclude that of Elizabeth and her kin.[141] Henry further convinced Parliament to backdate his reign to the day before the battle,[118] enabling him retrospectively to declare as traitors those who had fought against him at Bosworth Field.[142] Northumberland, who had remained inactive during the battle, was imprisoned but later released and reinstated to pacify the north in Henry's name.[143] Henry proved prepared to accept those who submitted to him regardless of their former allegiances.[144]

Of his supporters, Henry rewarded the Stanleys the most generously.[63] Aside from making William his chamberlain, he bestowed the earldom of Derby upon Lord Stanley along with grants and offices in other estates.[145] Henry rewarded Oxford by restoring to him the lands and titles confiscated by the Yorkists and appointing him as Constable of the Tower and admiral of England, Ireland, and Aquitaine. For his kin, Henry created Jasper Tudor the Duke of Bedford.[146] He returned to his mother the lands and grants stripped from her by Richard, and proved to be a filial son, granting her a place of honour in the palace and faithfully attending to her throughout his reign. Parliament's declaration of Margaret as femme sole effectively empowered her; she no longer needed to manage her estates through Stanley.[147] Elton points out that despite his initial largesse, Henry's supporters at Bosworth would enjoy his special favour for only the short term; in later years, he would instead promote those who best served his interests.[148]

Like the kings before him, Henry faced dissenters. The first open revolt occurred two years after Bosworth Field; Lambert Simnel claimed to be Edward Plantagenet, 17th Earl of Warwick, who was Edward IV's nephew. The Earl of Lincoln backed him for the throne and led rebel forces in the name of the House of York.[143] The rebel army fended off several attacks by Northumberland's forces, before engaging Henry's army at the Battle of Stoke Field on 16 June 1487.[145] Oxford and Bedford led Henry's men,[149] including several former supporters of Richard III.[150] Henry won this battle easily, but other malcontents and conspiracies would follow.[151] A rebellion in 1489 started with Northumberland's murder; military historian Michael C. C. Adams says that the author of a note, which was left next to Northumberland's body, blamed the earl for Richard's death.[118]

Legacy and historical significance

Contemporary accounts of the Battle of Bosworth can be found in four main sources, one of which is the English Croyland Chronicle, written by a senior Yorkist chronicler who relied on second-hand information from nobles and soldiers.[152] The other accounts were written by foreigners—Vergil, Jean Molinet, and Diego de Valera.[153] Whereas Molinet was sympathetic to Richard,[154] Vergil was in Henry's service and drew information from the king and his subjects to portray them in a good light.[155] Diego de Valera, whose information Ross regards as unreliable,[103] compiled his work from letters of Spanish merchants.[154] However, other historians have used Valera's work to deduce possibly valuable insights not readily evident in other sources.[156] Ross finds the poem, The Ballad of Bosworth Field, a useful source to ascertain certain details of the battle. The multitude of different accounts, mostly based on second- or third-hand information, has proved an obstacle to historians as they try to reconstruct the battle.[103] Their common complaint is that, except for its outcome, very few details of the battle are found in the chronicles. According to historian Michael Hicks, the Battle of Bosworth is one of the worst-recorded clashes of the Wars of the Roses.[102]

Historical depictions and interpretations

Henry tried to present his victory as a new beginning for the country;[157] he hired chroniclers to portray his reign as a \"modern age\" with its dawn in 1485.[158] Hicks states that the works of Vergil and the blind historian Bernard André, promoted by subsequent Tudor administrations, became the authoritative sources for writers for the next four hundred years.[159] As such, Tudor literature paints a flattering picture of Henry's reign, depicting the Battle of Bosworth as the final clash of the civil war and downplaying the subsequent uprisings.[102] For England the Middle Ages ended in 1485, and English Heritage claims that other than William the Conqueror's successful invasion of 1066, no other year holds more significance in English history. By portraying Richard as a hunchbacked tyrant who usurped the throne by killing his nephews, the Tudor historians attached a sense of myth to the battle: it became an epic clash between good and evil with a satisfying moral outcome.[160] According to Reader Colin Burrow, André was so overwhelmed by the historic significance of the battle that he represented it with a blank page in his Henry VII (1502).[161] For Professor Peter Saccio, the battle was indeed a unique clash in the annals of English history, because \"the victory was determined, not by those who fought, but by those who delayed fighting until they were sure of being on the winning side.\"[61]

Historians such as Adams and Horrox believe that Richard lost the battle not for any mythic reasons, but because of morale and loyalty problems in his army. Most of the common soldiers found it difficult to fight for a liege whom they distrusted, and some lords believed that their situation might improve if Richard were dethroned.[108][150] According to Adams, against such duplicities Richard's desperate charge was the only knightly behaviour on the field. As fellow historian Michael Bennet puts it, the attack was \"the swan-song of [mediaeval] English chivalry\".[118] Adams believes this view was shared at the time by the printer William Caxton, who enjoyed sponsorship from Edward IV and Richard III. Nine days after the battle, Caxton published Thomas Malory's story about chivalry and death by betrayal—Le Morte d'Arthur—seemingly as a response to the circumstances of Richard's death.[118]

Elton does not believe Bosworth Field has any true significance, pointing out that the 20th-century English public largely ignored the battle until its quincentennial celebration. In his view, the dearth of specific information about the battle—no-one even knows exactly where it took place—demonstrates its insignificance to English society. Elton considers the battle as just one part of Henry's struggles to establish his reign, underscoring his point by noting that the young king had to spend ten more years pacifying factions and rebellions to secure his throne.[162]

Mackie asserts that, in hindsight, Bosworth Field is notable as the decisive battle that established a dynasty which would rule unchallenged over England for more than a hundred years.[163] Mackie notes that contemporary historians of that time, wary of the three royal successions during the long Wars of the Roses, considered Bosworth Field just another in a lengthy series of such battles. It was through the works and efforts of Francis Bacon and his successors that the public started to believe the battle had decided their futures by bringing about \"the fall of a tyrant\".[164]

Shakespearean dramatisation

William Shakespeare gives prominence to the Battle of Bosworth in his play, Richard III. It is the \"one big battle\"; no other fighting scene distracts the audience from this action,[165] represented by a one-on-one sword fight between Henry Tudor and Richard III.[166] Shakespeare uses their duel to bring a climactic end to the play and the Wars of the Roses; he also uses it to champion morality, portraying the \"unequivocal triumph of good over evil\".[167] Richard, the villainous lead character, has been built up in the battles of Shakespeare's earlier play, Henry VI, Part 3, as a \"formidable swordsman and a courageous military leader\"—in contrast to the dastardly means by which he becomes king in Richard III.[168] Although the Battle of Bosworth has only five sentences to direct it, three scenes and more than four hundred lines precede the action, developing the background and motivations for the characters in anticipation of the battle.[167]

Shakespeare's account of the battle was mostly based on chroniclers Edward Hall's and Raphael Holinshed's dramatic versions of history, which were sourced from Vergil's chronicle. However, Shakespeare's attitude towards Richard was shaped by scholar Thomas More, whose writings displayed extreme bias against the Yorkist king.[169] The result of these influences is a script that vilifies the king, and Shakespeare had few qualms about departing from history to incite drama.[170] Margaret of Anjou died in 1482, but Shakespeare had her speak to Richard's mother before the battle to foreshadow Richard's fate and fulfill the prophecy she had given in Henry VI.[171] Shakespeare exaggerated the cause of Richard's restless night before the battle, imagining it as a haunting by the ghosts of those whom the king had murdered, including Buckingham.[172] Richard is portrayed as suffering a pang of conscience, but as he speaks he regains his confidence and asserts that he will be evil, if such needed to retain his crown.[173]

The fight between the two armies is simulated by rowdy noises made off-stage (alarums or alarms) while actors walk on-stage, deliver their lines, and exit. To build anticipation for the duel, Shakespeare requests more alarums after Richard's councillor, William Catesby, announces that the king is \"[enacting] more wonders than a man\". Richard punctuates his entrance with the classic line, \"A horse, a horse! My kingdom for a horse!\"[166] He refuses to withdraw, continuing to seek to slay Henry's doubles until he has killed his nemesis. There is no documentary evidence that Henry had five decoys at Bosworth Field; the idea was Shakespeare's invention. He drew inspiration from Henry IV's use of them at the Battle of Shrewsbury (1403) to amplify the perception of Richard's courage on the battlefield.[174] Similarly, the single combat between Henry and Richard is Shakespeare's creation. The True Tragedy of Richard III, by an unknown playwright, earlier than Shakespeare's, has no signs of staging such an encounter: its stage directions give no hint of visible combat.[175]

Despite the dramatic licences taken, Shakespeare's version of the Battle of Bosworth was the model of the event for English textbooks for many years during the 18th and 19th centuries.[176] This glamorised version of history, promulgated in books and paintings and played out on stages across the country, perturbed humorist Gilbert Abbott à Beckett.[177] He voiced his criticism in the form of a poem, equating the romantic view of the battle to watching a \"fifth-rate production of Richard III\": shabbily costumed actors fight the Battle of Bosworth on-stage while those with lesser roles lounge at the back, showing no interest in the proceedings.[178]

In Laurence Olivier's 1955 film adaptation of Richard III, the Battle of Bosworth is represented not by a single duel but a general melee that became the film's most recognised scene and a regular screening at Bosworth Battlefield Heritage Centre.[179] The film depicts the clash between the Yorkist and Lancastrian armies on an open field, focusing on individual characters amidst the savagery of hand-to-hand fighting, and received accolades for the realism portrayed.[180] One reviewer for The Manchester Guardian newspaper, however, was not impressed, finding the number of combatants too sparse for the wide plains and a lack of subtlety in Richard's death scene.[181] The means by which Richard is shown to prepare his army for the battle also earned acclaim. As Richard speaks to his men and draws his plans in the sand using his sword, his units appear on-screen, arraying themselves according to the lines that Richard had drawn. Intimately woven together, the combination of pictorial and narrative elements effectively turns Richard into a storyteller, who acts out the plot he has constructed.[182] Shakespearian critic Herbert Coursen extends that imagery: Richard sets himself up as a creator of men, but dies amongst the savagery of his creations. Coursen finds the depiction a contrast to that of Henry V and his \"band of brothers\".[183]

The adaptation of the setting for Richard III to a 1930s fascist England in Ian McKellen's 1995 film, however, did not sit well with historians. Adams posits that the original Shakespearian setting for Richard's fate at Bosworth teaches the moral of facing one's fate, no matter how unjust it is, \"nobly and with dignity\".[184] By overshadowing the dramatic teaching with special effects, McKellen's film reduces its version of the battle to a pyrotechnic spectacle about the death of a one-dimensional villain.[185] Coursen agrees that, in this version, the battle and Richard's end are trite and underwhelming.[186]

Battlefield location

The site of the battle is deemed by Leicestershire County Council to be in the vicinity of the town of Market Bosworth.[187] The council engaged historian Daniel Williams to research the battle, and in 1974 his findings were used to build the Bosworth Battlefield Heritage Centre and the presentation it houses.[188] Williams's interpretation, however, has since been questioned. Sparked by the battle's quincentenary celebration in 1985,[187] a dispute among historians has led many to doubt the accuracy of Williams's theory.[189][190] In particular, geological surveys conducted from 2003 to 2009 by the Battlefields Trust, a charitable organisation that protects and studies old English battlefields, show that the southern and eastern flanks of Ambion Hill were solid ground in the 15th century, contrary to Williams's claim that it was a large area of marshland.[191] Landscape archaeologist Glenn Foard, leader of the survey,[192] said the collected soil samples and finds of medieval military equipment suggest that the battle took place two miles (3.2 km) south-west of Ambion Hill (52°34′41″N 1°26′02″W),[193] contrary to the popular belief that it was fought near the foot of the hill.[194]

Historians' theories

English Heritage argues that the battle was named after Market Bosworth because the town was then the nearest significant settlement to the battlefield.[155] As explored by Professor Philip Morgan, a battle might initially not be named specifically at all. As time passes, writers of administrative and historical records find it necessary to identify a notable battle, ascribing it a name that is usually toponymical in nature and sourced from combatants or observers. This name then becomes accepted by society and without question.[195] Early records associated the Battle of Bosworth with \"Brownehethe\", \"bellum Miravallenses\", \"Sandeford\" and \"Dadlyngton field\".[196] The earliest record, a municipal memorandum of 23 August 1485 from York,[197] locates the battle \"on the field of Redemore\".[198] This is corroborated by a 1485–86 letter that mentions \"Redesmore\" as its site.[188] According to the historian, Peter Foss, records did not associate the battle with \"Bosworth\" until 1510.[196]

Foss is named by English Heritage as the principal advocate for \"Redemore\" as the battle site. He suggests the name is derived from \"Hreod Mor\", an Anglo-Saxon phrase that means \"reedy marshland\". Basing his opinion on 13th- and 16th-century church records, he believes \"Redemore\" was an area of wetland that lay between Ambion Hill and the village of Dadlington, and was close to the Fenn Lanes, a Roman road running east to west across the region.[188] Foard believes this road to be the most probable route that both armies took to reach the battlefield.[199] Williams dismisses the notion of \"Redmore\" as a specific location, saying that the term refers to a large area of reddish soil; Foss argues that Williams's sources are local stories and flawed interpretations of records.[200] Moreover, he proposes that Williams was influenced by William Hutton's 1788 The Battle of Bosworth-Field, which Foss blames for introducing the notion that the battle was fought west of Ambion Hill on the north side of the River Sence.[201] Hutton, as Foss suggests, misinterpreted a passage from his source, Raphael Holinshed's 1577 Chronicle. Holinshed wrote, \"King Richard pitched his field on a hill called Anne Beame, refreshed his soldiers and took his rest.\" Foss believes that Hutton mistook \"field\" to mean \"field of battle\", thus creating the idea that the fight took place on Anne Beame (Ambion) Hill. To \"[pitch] his field\", as Foss clarifies, was a period expression for setting up a camp.[202]

Foss brings further evidence for his \"Redemore\" theory by quoting Edward Hall's 1550 Chronicle. Hall stated that Richard's army stepped onto a plain after breaking camp the next day. Furthermore, historian William Burton, author of Description of Leicestershire (1622),[188] wrote that the battle was \"fought in a large, flat, plaine, and spacious ground, three miles [5 km] distant from [Bosworth], between the Towne of Shenton, Sutton [Cheney], Dadlington and Stoke [Golding]\".[200] In Foss's opinion both sources are describing an area of flat ground north of Dadlington.[203]

Physical site

English Heritage, responsible for managing England's historic sites, used both theories to designate the site for Bosworth Field. Without preference for either theory, they constructed a single continuous battlefield boundary that encompasses the locations proposed by both Williams and Foss.[204] The region has experienced extensive changes over the years, starting after the battle. Holinshed stated in his chronicle that he found firm ground where he expected the marsh to be, and Burton confirmed that by the end of the 16th century, areas of the battlefield were enclosed and had been improved to make them agriculturally productive. Trees were planted on the south side of Ambion Hill, forming Ambion Wood. In the 18th and 19th centuries, the Ashby Canal carved through the land west and south-west of Ambion Hill. Winding alongside the canal at a distance, the Ashby and Nuneaton Joint Railway crossed the area on an embankment.[155][205] The changes to the landscape were so extensive that when Hutton revisited the region in 1807 after an earlier 1788 visit, he could not readily find his way around.[155]

Richard's Well, where the last Yorkist king supposedly took a drink of water on the day of the battle
Bosworth Battlefield Heritage Centre was built on Ambion Hill, near Richard's Well. According to legend, Richard III drank from one of the several springs in the region on the day of the battle.[206] In 1788, a local pointed out one of the springs to Hutton as the one mentioned in the legend.[130] A stone structure was later built over the location. The inscription on the well reads:

Near this spot, on August 22nd 1485, at the age of 32, King Richard III fell fighting gallantly in defence of his realm & his crown against the usurper Henry Tudor.

The Cairn was erected by Dr. Samuel Parr in 1813 to mark the well from which the king is said to have drunk during the battle.

It is maintained by the Fellowship of the White Boar.[207]

North-west of Ambion Hill, just across the northern tributary of the Sence, a flag and memorial stone mark Richard's Field. Erected in 1973, the site was selected on the basis of Williams's theory.[208] St James's Church at Dadlington is the only structure in the area that is reliably associated with the Battle of Bosworth; the bodies of those killed in the battle were buried there.[130]

Rediscovered battlefield and possible battle scenario

The very extensive survey carried out (2005–2009) by the Battlefields Trust headed by Glenn Foard led eventually to the discovery of the real location of the core battlefield.[209] This lies about a kilometre further west of the location suggested by Peter Foss. It is in what was at the time of the battle an area of marginal land at the meeting of several township boundaries. There was a cluster of field names suggesting the presence of marshland and heath. Thirty four lead round shot[210] were discovered as a result of systematic metal detecting (more than the total found previously on all other C15th European battlefields), as well as other significant finds,[211] including a small silver gilt badge depicting a boar. Experts believe that the boar badge could indicate the actual site of Richard III's death, since this high-status badge depicting his personal emblem was probably worn by a member of his close retinue.[212]

Bosworth Battlefield (Fenn Lane Farm)

A new interpretation[213] of the battle now integrates the historic accounts with the battlefield finds and landscape history. The new site lies either side of the Fenn Lanes Roman road, close to Fenn Lane Farm and is some three kilometres to the south-west of Ambion Hill.

Based on the round shot scatter, the likely size of Richard III's army, and the topography, Glenn Foard and Anne Curry think that Richard may have lined up his forces on a slight ridge which lies just east of Fox Covert Lane and behind a postulated medieval marsh.[214][215] Richard's vanguard commanded by the Duke of Norfolk was on the right (north) side of Richard's battle line, with the Earl of Northumberland on Richard's left (south) side.

Tudor's forces approached along the line of the Roman road and lined up to the west of the present day Fenn Lane Farm, having marched from the vicinity of Merevale in Warwickshire.[216]

Historic England have re-defined the boundaries of the registered Bosworth Battlefield to incorporate the newly identified site. There are hopes that public access to the site will be possible in the future.[213][217]";
