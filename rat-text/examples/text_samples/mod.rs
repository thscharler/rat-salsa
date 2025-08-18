use rat_text::text_area::TextAreaState;
use rat_text::TextRange;
use ropey::{Rope, RopeBuilder};
use std::ops::Range;

pub fn add_range_styles(state: &mut TextAreaState, styles: Vec<(TextRange, usize)>) {
    for (range, style) in styles {
        _ = state.add_range_style(range, style);
    }
}

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

pub fn sample_scott_1() -> (Rope, Vec<(TextRange, usize)>) {
    (Rope::from_str(SCOTT_1), Vec::new())
}

pub fn sample_emoji() -> (Rope, Vec<(TextRange, usize)>) {
    (
        Rope::from_str("short text\nw🤷‍♂️x\nw🤷‍♀️x\nw🤦‍♂️x\nw❤️x\nw🤦‍♀️x\nw💕x\nw🙍🏿‍♀️x\n"),
        Vec::new(),
    )
}

pub fn sample_tabs() -> (Rope, Vec<(TextRange, usize)>) {
    (
        Rope::from_str("\t\ttabs\n\t\t\t\ttabs\n\tt\tt\tt\n"),
        Vec::new(),
    )
}

pub fn sample_lorem() -> (Rope, Vec<(Range<usize>, usize)>) {
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

pub fn sample_pattern_0() -> (Rope, Vec<(TextRange, usize)>) {
    (Rope::from_str(PATTERN_0), Vec::new())
}

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

Scott wurde als Sohn eines Berufssoldaten geboren. Sein Vater, den er selten zu sehen bekam, diente bei den Royal Engineers (Kampfunterstützungstruppen der britischen Armee) in hoher Position als Ingenieur und Transportkontrolleur. Nach Aufenthalten in Cumbria, Wales und Deutschland (dort zwischen 1947 und 1952 in Hamburg) ließ sich die Familie in Stockton-on-Tees im Norden Englands nieder (die industriell geprägte Landschaft inspirierte später Szenen in Blade Runner). Zum Ende seiner Kindheit und Jugend hatte er eigenen Angaben zufolge wegen der vielen Umzüge 10 Schulen besucht.[2]

Scott erlernte 1954 bis 1958 Grafikdesign und Malerei am West Hartlepool College of Art und erlangte das Diplom mit Auszeichnung. Er studierte daraufhin Grafikdesign (M.A., 1960 bis 1962) am Royal College of Art in London, wo David Hockney einer seiner Mitstudenten war. Er schloss 1963 mit Auszeichnung ab. Scott erhielt ein einjähriges Reisestipendium in die USA und wurde bei Time Life beschäftigt, wo er mit den Dokumentaristen Richard Leacock und D. A. Pennebaker arbeitete. Nach seiner Rückkehr nahm er 1965 eine Lehrstelle bei der BBC als Szenenbildner an. Diese Position führte ihn zur Mitarbeit an beliebten Fernsehproduktionen wie der Polizei-Serie Z-Cars oder der Science-Fiction-Serie Out of the Unknown. Nach kurzer Zeit wurde er ins Trainingsprogramm für Regisseure aufgenommen und inszenierte einige Episoden selbst.

1968 verließ Scott die BBC, um Ridley Scott Associates (RSA) zu gründen. An dem Projekt arbeiteten neben seinem Bruder Tony Regisseure wie Alan Parker, Hugh Hudson und Hugh Johnson mit. RSA wurde zu einem der erfolgreichsten Werbefilm-Häuser in Europa, in dessen Auftrag Scott für über 2000 Werbespots verantwortlich zeichnet; viele davon wurden auf den Festspielen von Cannes und Venedig ausgezeichnet.

Scott gilt in der Branche als ökonomischer Regisseur, da er in der Regel mit einem Drittel der Drehtage seiner Kollegen auskommt. Eigenen Worten zufolge verdankt er dies seiner Vergangenheit als Werbe- und Videospotregisseur sowie der Tatsache, dass er manche Szenen mit bis zu 15 Kameras gleichzeitig drehe.[3] Seit dem Jahr 2000, als sie in Gladiator eine Nebenrolle spielte, ist Scott mit der costa-ricanischen Schauspielerin Giannina Facio, Tochter des Diplomaten und Politikers Gonzalo Facio (1918–2018), liiert. Im Juni 2015 heiratete das Paar.[4]

Im Jahr 2003 wurde Scott von der britischen Königin aufgrund seiner Verdienste um die Kunst zum Ritter geschlagen, am 8. Mai 2024 ernannte Thronfolger Prinz William ihn zum Knight Grand Cross of the Order of the British Empire. Scott ist damit Träger des höchsten britischen Verdienstordens.

Sein jüngerer Bruder ist der Regisseur und Filmproduzent Tony Scott, der sich 2012 das Leben nahm. Seine Söhne Luke und Jake und seine Tochter Jordan sind ebenfalls im Filmgeschäft tätig.

Scott lebt in Los Angeles, besitzt aber seit etwa Anfang der 90er Jahre ein Haus in Südfrankreich.[2]

Werk

Scotts Markenzeichen ist ein ausgeprägt ästhetischer und malerischer visueller Stil, der sich durch seine jahrelange Erfahrung als Production Designer und Regisseur von Werbespots entwickelt hat. Zusammen mit seinem Bruder Tony betrieb er ab 1968 die Produktionsfirma für Werbefilme Ridley Scott Associates (RSA).

Scotts erster Themenfilm Die Duellisten (1977) war zwar kommerziell kein großer Erfolg, fand aber bei der Kritik genug Beachtung, um Scott die Realisierung des Science-Fiction-Films Alien – Das unheimliche Wesen aus einer fremden Welt (1979) zu ermöglichen. Sein nächster Film Blade Runner (1982), basierend auf dem Roman Träumen Androiden von elektrischen Schafen? von Philip K. Dick, spielt in einem düster-futuristischen Los Angeles. Das Werk war visuell derart beeindruckend, dass es für eine ganze Generation Cyberpunk-Literatur, -Musik und -Kunst als Inspiration diente. In der Folge drehte Scott Legende (1985), Der Mann im Hintergrund (1987) und Black Rain (1989), die alle nicht an die Bedeutung und den Erfolg der vorigen Werke anknüpfen konnten. Legende setzte sich jedoch im Lauf der Zeit als Fantasy-Kultfilm durch und wurde 2002 mit einem restaurierten Director’s Cut ergänzt.

Die von der Kritik stetig vorgebrachte Beschuldigung, visuellen Stil vor Inhalt und Charakterzeichnung zu stellen, wurde mit Thelma & Louise (1991) entkräftet. Neben guten Kritiken erhielt Scott seine erste Oscar-Nominierung für die beste Regie. Danach folgten mit dem Kolumbus-Film 1492 – Die Eroberung des Paradieses (1992), White Squall – Reißende Strömung (1996) und Die Akte Jane (1997) erneut Filme, die künstlerisch und kommerziell durchfielen. Insbesondere der Militärfilm Die Akte Jane, in dem Demi Moore eine Frau spielt, die als erste Mitglied bei den Navy Seals werden will, wurde wegen einer nach Ansicht vieler Kritiker undifferenzierten Pro-Militär-Haltung angegriffen. Mit Gladiator feierte Scott 2000 ein triumphales Comeback. Der Film war beim Publikum sehr erfolgreich und gewann neben dem Oscar für den besten Film im Jahr 2000 auch den Golden Globe 2001. Die Regie-Leistung wurde ebenfalls nominiert, den Preis erhielt Scott jedoch nicht. Eine weitere Oscar-Nominierung erhielt er für den kontroversen Kriegsfilm Black Hawk Down (2001), der einen verunglückten US-amerikanischen Militäreinsatz in Somalia thematisiert und in eindrucksvolle Bilder umsetzt. Black Hawk Down prägte die neuere Action-Darstellung und verhalf der dokumentaristischen Kameraführung zum Durchbruch in der Filmkunst.

Scott übernahm die Regie bei dem Film Hannibal (2001), der Fortsetzung zu Das Schweigen der Lämmer (1991). 2005/2006 folgte in zwei Versionen der Film Königreich der Himmel. 2006 erschien Ein gutes Jahr nach dem Roman Ein guter Jahrgang seines Landsmannes Peter Mayle. Er handelt von einem Bankmanager, der von seinem Onkel ein Weingut in der Provence erbt und daraufhin beschließt, sein Leben umzukrempeln. Die Hauptrolle spielt der neuseeländische Schauspieler Russell Crowe. Gemeinsam mit seinem Bruder Tony produzierte Scott für den amerikanischen Kabelsender TNT die Miniserie The Company – Im Auftrag der CIA, die im August 2007 ausgestrahlt wurde. The Company erzählt die Geschichte dreier Yale-Absolventen, die in der Nachkriegszeit auf Seiten der CIA bzw. des KGB in den Kalten Krieg verwickelt werden. In den Hauptrollen sind u. a. Chris O’Donnell, Michael Keaton und Alfred Molina zu sehen.

Im Oktober 2008 bestätigte Scott, dass er 25 Jahre warten musste, bis die Rechte an dem Buch Der Ewige Krieg von Joe Haldeman für eine Verfilmung zur Verfügung standen. [5] Scott plane, dieses Buch in 3D zu verfilmen.[6]
 
Für den US-Fernsehsender CBS produzierte Scott seit 2009 die Serie Good Wife. Die Ausstrahlung begann in den USA im September 2009, in Deutschland bei ProSieben Ende März 2010. Auch hier arbeitete er mit seinem Bruder Tony zusammen. Mit der 2009 abgedrehten Produktion Robin Hood legte Scott erneut einen Historienfilm vor. Mit seinem 22. Spielfilm, realisiert nach einem Drehbuch von Brian Helgeland mit Russell Crowe in der Titelrolle, wurden am 12. Mai 2010 die 63. Filmfestspiele von Cannes eröffnet.[7]

Scott arbeitete 2009 an der ersten Verfilmung von Aldous Huxleys Roman Schöne neue Welt für das Kino. Der Film sollte von ihm und Leonardo DiCaprio produziert werden, Drehbuchautor sollte Farhad Safinia sein. Scott sollte voraussichtlich auch Regie führen, der Film wurde jedoch bis heute nicht realisiert.[8] Der Film Prometheus war ursprünglich als Prequel zu Scotts erstem großen Erfolg Alien geplant. Das Drehbuch stammt von Jon Spaihts; Damon Lindelof überarbeitete das Drehbuch für 20th Century Fox. In den USA erfolgte der Kinostart am 8. Juni 2012. 2017 folgte die Fortsetzung Alien: Covenant. Im selben Jahr verfilmte Scott mit Alles Geld der Welt den Entführungsfall um John Paul Getty III. Im Zuge des Skandals um Kevin Spacey, der ab Ende Oktober 2017 mit Vorwürfen der sexuellen Belästigung konfrontiert wurde, entschloss sich das Filmteam und Sony Pictures, alle Szenen mit Spacey aus dem Film zu schneiden. Scott musste diese Szenen kurzfristig mit Christopher Plummer nachdrehen. ";
