use crate::mini_salsa::{run_ui, setup_logging, MiniSalsaState};
#[allow(unused_imports)]
use rat_event::{ct_event, try_flow, Outcome};
use rat_scrolled::Scroll;
use rat_text::text_area::{TextArea, TextAreaState};
use rat_text::{text_area, TextRange};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Style, Stylize};
use ratatui::widgets::{Block, Paragraph, StatefulWidget};
use ratatui::Frame;
use ropey::RopeBuilder;
use std::fmt;

mod mini_salsa;

fn main() -> Result<(), anyhow::Error> {
    setup_logging()?;

    let mut data = Data {};

    let mut state = State {
        info: true,
        textarea: Default::default(),
    };
    state.textarea.set_auto_indent(false);
    insert_text_1(&mut state);

    run_ui(handle_input, repaint_input, &mut data, &mut state)
}

struct Data {}

struct State {
    pub(crate) info: bool,
    pub(crate) textarea: TextAreaState,
}

fn repaint_input(
    frame: &mut Frame<'_>,
    area: Rect,
    _data: &mut Data,
    istate: &mut MiniSalsaState,
    state: &mut State,
) -> Result<(), anyhow::Error> {
    let l1 = Layout::vertical([
        Constraint::Length(1),
        Constraint::Fill(1),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .split(area);

    let l2 = Layout::horizontal([
        Constraint::Length(15),
        Constraint::Fill(1),
        Constraint::Length(1),
        Constraint::Length(25),
    ])
    .split(l1[1]);

    TextArea::new()
        .block(Block::bordered().style(Style::default().gray().on_dark_gray()))
        .scroll(
            Scroll::new()
                .scroll_by(1)
                .style(Style::default().gray().on_dark_gray()),
        )
        .set_horizontal_max_offset(256)
        .style(Style::default().white().on_dark_gray())
        .select_style(Style::default().black().on_yellow())
        .text_style([
            Style::new().red(),
            Style::new().underlined(),
            Style::new().green(),
            Style::new().on_yellow(),
        ])
        .render(l2[1], frame.buffer_mut(), &mut state.textarea);

    if let Some((cx, cy)) = state.textarea.screen_cursor() {
        frame.set_cursor_position((cx, cy));
    }

    if state.info {
        use fmt::Write;
        let mut stats = String::new();
        _ = writeln!(&mut stats);
        _ = writeln!(&mut stats, "cursor: {:?}", state.textarea.cursor(),);
        _ = writeln!(&mut stats, "anchor: {:?}", state.textarea.anchor());
        if let Some((scx, scy)) = state.textarea.screen_cursor() {
            _ = writeln!(&mut stats, "screen: {}:{}", scx, scy);
        } else {
            _ = writeln!(&mut stats, "screen: None",);
        }
        _ = writeln!(
            &mut stats,
            "width: {:?} ",
            state.textarea.line_width(state.textarea.cursor().y)
        );
        _ = writeln!(
            &mut stats,
            "next word: {:?} {:?}",
            state.textarea.next_word_start(state.textarea.cursor()),
            state.textarea.next_word_end(state.textarea.cursor())
        );
        _ = writeln!(
            &mut stats,
            "prev word: {:?} {:?}",
            state.textarea.prev_word_start(state.textarea.cursor()),
            state.textarea.prev_word_end(state.textarea.cursor())
        );

        _ = write!(&mut stats, "cursor-styles: ",);
        let mut styles = Vec::new();
        let cursor_byte = state.textarea.byte_at(state.textarea.cursor());
        state.textarea.styles_at(cursor_byte.start, &mut styles);
        for s in styles {
            _ = write!(&mut stats, "{}, ", s);
        }
        _ = writeln!(&mut stats);

        if let Some(st) = state.textarea.value.styles() {
            _ = writeln!(&mut stats, "text-styles: {}", st.count());
        }
        if let Some(st) = state.textarea.value.styles() {
            for r in st.take(20) {
                _ = writeln!(&mut stats, "    {:?}", r);
            }
        }
        let dbg = Paragraph::new(stats);
        frame.render_widget(dbg, l2[3]);
    }

    let ccursor = state.textarea.selection();
    istate.status[0] = format!(
        "{}:{} - {}:{}",
        ccursor.start.y, ccursor.start.x, ccursor.end.y, ccursor.end.x,
    );

    Ok(())
}

fn handle_input(
    event: &crossterm::event::Event,
    _data: &mut Data,
    _istate: &mut MiniSalsaState,
    state: &mut State,
) -> Result<Outcome, anyhow::Error> {
    try_flow!(text_area::handle_events(&mut state.textarea, true, event));

    try_flow!(match event {
        ct_event!(key press ALT-'0') => {
            state.info = !state.info;
            Outcome::Changed
        }
        ct_event!(key press ALT-'1') => insert_text_0(state),
        ct_event!(key press ALT-'2') => insert_text_1(state),
        ct_event!(key press ALT-'3') => insert_text_2(state),
        ct_event!(key press ALT-'4') => insert_text_3(state),
        _ => Outcome::Continue,
    });

    Ok(Outcome::Continue)
}

pub(crate) fn insert_text_3(state: &mut State) -> Outcome {
    let l = lorem_rustum::LoremRustum::new(1_000_000);

    let mut style = Vec::new();

    let mut buf = RopeBuilder::new();
    let mut pos = 0;
    let mut width = 0;
    for p in l.body {
        buf.append(p);
        buf.append(" ");
        width += p.len() + 1;

        if p == "macro" {
            style.push((pos..pos + p.len(), 0));
        } else if p == "assert!" {
            style.push((pos..pos + p.len(), 1));
        } else if p == "<'a>" {
            style.push((pos..pos + p.len(), 2));
        } else if p == "await" {
            style.push((pos..pos + p.len(), 3));
        }

        pos += p.len() + 1;

        if width > 66 {
            buf.append("\n");
            width = 0;
            pos += 1;
        }
    }
    let buf = buf.finish();

    state.textarea.set_rope(buf);
    state.textarea.set_styles(style);

    Outcome::Changed
}

pub(crate) fn insert_text_2(state: &mut State) -> Outcome {
    state.textarea.set_text("");
    Outcome::Changed
}

pub(crate) fn insert_text_1(state: &mut State) -> Outcome {
    let str = "short text\nw🤷‍♂️x\nw🤷‍♀️x\nw🤦‍♂️x\nw❤️x\nw🤦‍♀️x\nw💕x\nw🙍🏿‍♀️x\n";
    // let str = "\t\ttabs\n\t\t\t\ttabs\n\tt\tt\tt\n";
    state.textarea.set_text(str);
    Outcome::Changed
}

pub(crate) fn insert_text_0(state: &mut State) -> Outcome {
    state.textarea.set_text(DATA_0);

    state
        .textarea
        .add_range_style(TextRange::new((0, 0), (13, 0)), 0)
        .unwrap();
    state
        .textarea
        .add_range_style(TextRange::new((0, 1), (13, 1)), 0)
        .unwrap();
    state
        .textarea
        .add_range_style(TextRange::new((4, 3), (17, 3)), 0)
        .unwrap();
    state
        .textarea
        .add_range_style(TextRange::new((31, 44), (44, 44)), 0)
        .unwrap();

    // overlapping styles
    state
        .textarea
        .add_range_style(TextRange::new((30, 7), (42, 7)), 0)
        .unwrap();
    state
        .textarea
        .add_range_style(TextRange::new((37, 7), (41, 7)), 1)
        .unwrap();

    state
        .textarea
        .add_range_style(TextRange::new((44, 7), (63, 7)), 0)
        .unwrap();
    state
        .textarea
        .add_range_style(TextRange::new((58, 7), (62, 7)), 1)
        .unwrap();

    state
        .textarea
        .add_range_style(TextRange::new((65, 7), (6, 8)), 0)
        .unwrap();
    state
        .textarea
        .add_range_style(TextRange::new((1, 8), (5, 8)), 0)
        .unwrap();

    state
        .textarea
        .add_range_style(TextRange::new((8, 8), (24, 8)), 0)
        .unwrap();
    state
        .textarea
        .add_range_style(TextRange::new((19, 8), (23, 8)), 0)
        .unwrap();

    state
        .textarea
        .add_range_style(TextRange::new((26, 8), (48, 8)), 0)
        .unwrap();
    state
        .textarea
        .add_range_style(TextRange::new((43, 8), (47, 8)), 0)
        .unwrap();

    state
        .textarea
        .add_range_style(TextRange::new((53, 8), (73, 8)), 0)
        .unwrap();
    state
        .textarea
        .add_range_style(TextRange::new((68, 8), (72, 8)), 0)
        .unwrap();

    Outcome::Changed
}

static DATA_0: &str = "Ridley Scott
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
