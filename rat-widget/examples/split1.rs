#![allow(dead_code)]

use crate::mini_salsa::endless_scroll::{EndlessScroll, EndlessScrollState};
use crate::mini_salsa::theme::THEME;
use crate::mini_salsa::{run_ui, setup_logging, MiniSalsaState};
use rat_event::{ct_event, try_flow, ConsumedEvent, HandleEvent, Regular};
use rat_focus::{Focus, FocusBuilder, HasFocus};
use rat_menu::event::MenuOutcome;
use rat_menu::menuline::{MenuLine, MenuLineState};
use rat_scrolled::Scroll;
use rat_widget::event::Outcome;
use rat_widget::paragraph::{Paragraph, ParagraphState};
use rat_widget::splitter::{Split, SplitResize, SplitState, SplitType};
use rat_widget::statusline::StatusLineState;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Style, Stylize};
use ratatui::text::Line;
use ratatui::widgets::{Block, BorderType, StatefulWidget, Widget, Wrap};
use ratatui::Frame;

mod mini_salsa;

fn main() -> Result<(), anyhow::Error> {
    setup_logging()?;

    let mut data = Data {};

    let mut state = State {
        dir: Direction::Horizontal,
        split_type: Default::default(),
        border_type: None,
        inner_border_type: None,
        resize: Default::default(),
        focus: Default::default(),
        split: Default::default(),
        left: Default::default(),
        right: Default::default(),
        right_right: Default::default(),
        menu: Default::default(),
        status: Default::default(),
    };
    state.menu.focus.set(true);

    run_ui(
        "split1",
        |_| {},
        handle_input,
        repaint_input,
        &mut data,
        &mut state,
    )
}

struct Data {}

struct State {
    dir: Direction,
    split_type: SplitType,
    border_type: Option<BorderType>,
    inner_border_type: Option<BorderType>,
    resize: SplitResize,

    focus: Focus,

    split: SplitState,
    left: ParagraphState,
    right: EndlessScrollState,
    right_right: EndlessScrollState,
    menu: MenuLineState,
    status: StatusLineState,
}

fn repaint_input(
    frame: &mut Frame<'_>,
    area: Rect,
    _data: &mut Data,
    _istate: &mut MiniSalsaState,
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
        Constraint::Length(25),
        Constraint::Fill(1),
        Constraint::Length(15),
    ])
    .split(l1[1]);

    // create split widget.
    let mut split = Split::new()
        .styles(THEME.split_style())
        .direction(state.dir)
        .split_type(state.split_type)
        .resize(state.resize)
        .mark_offset(1)
        .constraints([
            Constraint::Fill(1),
            Constraint::Fill(1),
            Constraint::Fill(1),
        ]);
    if let Some(blk) = state.border_type {
        split = split
            .block(
                Block::bordered()
                    .border_type(blk) //
                    .border_style(THEME.block()),
            )
            .join_1(blk)
            .join_0(blk);
    }
    let (split, split_areas) = split.into_widget_layout(l2[1], &mut state.split);

    // First split widget. Show some TEXT.
    if !state.split.is_hidden(0) {
        let mut w_left = Paragraph::new(TEXT)
            .styles(THEME.paragraph_style())
            .wrap(Wrap::default());
        if let Some(inner_border) = state.inner_border_type {
            // configurable border
            w_left = w_left.block(
                Block::bordered()
                    .title("inner block")
                    .border_style(THEME.magenta(0))
                    .border_type(inner_border),
            );
        }
        let mut scroll_left = Scroll::new().styles(THEME.scroll_style());
        if state.dir == Direction::Horizontal {
            // don't start the scrollbar at the top of the area, start it 3 below.
            // leaves some space for the split handles.
            scroll_left = scroll_left.start_margin(3);
        }
        w_left = w_left.vscroll(scroll_left);
        w_left.render(split_areas[0], frame.buffer_mut(), &mut state.left);
    }

    // some dummy widget
    EndlessScroll::new()
        .max(100000) //
        .style(THEME.deepblue(0))
        .focus_style(THEME.focus())
        .v_scroll(
            Scroll::new()
                .start_margin(3) //
                .styles(THEME.scroll_style()),
        )
        .render(split_areas[1], frame.buffer_mut(), &mut state.right);

    // some dummy widget
    EndlessScroll::new()
        .max(2024) //
        .style(THEME.bluegreen(0))
        .focus_style(THEME.focus())
        .v_scroll(
            Scroll::new() //
                .styles(THEME.scroll_style()),
        )
        .render(split_areas[2], frame.buffer_mut(), &mut state.right_right);

    // Render split after all the content.
    split.render(l2[1], frame.buffer_mut(), &mut state.split);

    // render layout detail info
    let mut area = Rect::new(l2[0].x, l2[0].y, l2[0].width, 1);
    Line::from("F1: hide first")
        .yellow()
        .render(area, frame.buffer_mut());
    area.y += 1;
    Line::from("F3: toggle")
        .yellow()
        .render(area, frame.buffer_mut());
    area.y += 1;
    Line::from("F4: type")
        .yellow()
        .render(area, frame.buffer_mut());
    area.y += 1;
    Line::from("F5: border")
        .yellow()
        .render(area, frame.buffer_mut());
    area.y += 1;
    Line::from("F6: left border")
        .yellow()
        .render(area, frame.buffer_mut());
    area.y += 1;
    Line::from("F7: resize")
        .yellow()
        .render(area, frame.buffer_mut());
    area.y += 1;
    Line::from("F12: key-nav")
        .yellow()
        .render(area, frame.buffer_mut());
    area.y += 1;
    area.y += 1;

    Line::from(format!(
        "area {},{}+{}+{}",
        state.split.inner.x, state.split.inner.y, state.split.inner.width, state.split.inner.height
    ))
    .render(area, frame.buffer_mut());
    area.y += 1;
    Line::from("areas").render(area, frame.buffer_mut());
    area.y += 1;
    for a in &split_areas {
        Line::from(format!("{},{}+{}+{}", a.x, a.y, a.width, a.height))
            .render(area, frame.buffer_mut());
        area.y += 1;
    }

    use std::fmt::Write;
    let txt = state
        .split
        .area_lengths()
        .iter()
        .fold(String::from("Length "), |mut v, w| {
            _ = write!(v, "{}, ", *w);
            v
        });
    Line::from(txt).render(area, frame.buffer_mut());
    area.y += 1;
    Line::from(format!("Drag {:?}", state.split.mouse.drag.get())).render(area, frame.buffer_mut());
    area.y += 1;
    Line::from(format!("Mark {:?}", state.split.focus_marker)).render(area, frame.buffer_mut());
    area.y += 1;
    Line::from(format!("{:?}", state.split.split_type)).render(area, frame.buffer_mut());
    area.y += 1;

    let menu1 = MenuLine::new()
        .title("||||")
        .item_parsed("_Quit")
        .title_style(Style::default().black().on_yellow())
        .style(Style::default().black().on_dark_gray());
    frame.render_stateful_widget(menu1, l1[3], &mut state.menu);

    Ok(())
}

// handle focus
fn focus(state: &mut State) -> Focus {
    // rebuild, using the old focus for storage and to reset
    // widgets no longer in the focus loop.
    let mut builder = FocusBuilder::new(None);
    builder.widget(&state.split);
    if !state.split.is_hidden(0) {
        builder.widget(&state.left);
    }
    builder.widget(&state.right);
    builder.widget(&state.right_right);
    builder.widget(&state.menu);
    builder.build()
}

fn handle_input(
    event: &crossterm::event::Event,
    _data: &mut Data,
    istate: &mut MiniSalsaState,
    state: &mut State,
) -> Result<Outcome, anyhow::Error> {
    state.focus = focus(state);

    // handle focus events
    istate.focus_outcome = state.focus.handle(event, Regular);

    try_flow!(match event {
        ct_event!(keycode press F(1)) => {
            if state.split.is_hidden(0) {
                state.split.show_split(0);
            } else {
                state.split.hide_split(0);
            }
            Outcome::Changed
        }
        ct_event!(keycode press F(3)) => {
            if state.dir == Direction::Horizontal {
                state.dir = Direction::Vertical;
            } else {
                state.dir = Direction::Horizontal;
            }
            Outcome::Changed
        }
        ct_event!(keycode press F(4)) => {
            state.split_type = match state.split_type {
                SplitType::FullEmpty => SplitType::Scroll,
                SplitType::Scroll => SplitType::Widget,
                SplitType::Widget => SplitType::FullPlain,
                SplitType::FullPlain => SplitType::FullDouble,
                SplitType::FullDouble => SplitType::FullThick,
                SplitType::FullThick => SplitType::FullQuadrantInside,
                SplitType::FullQuadrantInside => SplitType::FullQuadrantOutside,
                SplitType::FullQuadrantOutside => SplitType::FullEmpty,
            };
            Outcome::Changed
        }
        ct_event!(keycode press SHIFT-F(4)) => {
            state.split_type = match state.split_type {
                SplitType::FullEmpty => SplitType::FullQuadrantOutside,
                SplitType::Scroll => SplitType::FullEmpty,
                SplitType::Widget => SplitType::Scroll,
                SplitType::FullPlain => SplitType::Widget,
                SplitType::FullDouble => SplitType::FullPlain,
                SplitType::FullThick => SplitType::FullDouble,
                SplitType::FullQuadrantInside => SplitType::FullThick,
                SplitType::FullQuadrantOutside => SplitType::FullQuadrantInside,
            };
            Outcome::Changed
        }
        ct_event!(keycode press F(5)) => {
            state.border_type = match state.border_type {
                None => Some(BorderType::Plain),
                Some(BorderType::Plain) => Some(BorderType::Double),
                Some(BorderType::Double) => Some(BorderType::Rounded),
                Some(BorderType::Rounded) => Some(BorderType::Thick),
                Some(BorderType::Thick) => Some(BorderType::QuadrantInside),
                Some(BorderType::QuadrantInside) => Some(BorderType::QuadrantOutside),
                Some(BorderType::QuadrantOutside) => None,
            };
            Outcome::Changed
        }
        ct_event!(keycode press SHIFT-F(5)) => {
            state.border_type = match state.border_type {
                None => Some(BorderType::QuadrantOutside),
                Some(BorderType::Plain) => None,
                Some(BorderType::Double) => Some(BorderType::Plain),
                Some(BorderType::Rounded) => Some(BorderType::Double),
                Some(BorderType::Thick) => Some(BorderType::Rounded),
                Some(BorderType::QuadrantInside) => Some(BorderType::Thick),
                Some(BorderType::QuadrantOutside) => Some(BorderType::QuadrantInside),
            };
            Outcome::Changed
        }
        ct_event!(keycode press F(6)) => {
            state.inner_border_type = match state.inner_border_type {
                None => Some(BorderType::Plain),
                Some(BorderType::Plain) => Some(BorderType::Double),
                Some(BorderType::Double) => Some(BorderType::Rounded),
                Some(BorderType::Rounded) => Some(BorderType::Thick),
                Some(BorderType::Thick) => Some(BorderType::QuadrantInside),
                Some(BorderType::QuadrantInside) => Some(BorderType::QuadrantOutside),
                Some(BorderType::QuadrantOutside) => None,
            };
            Outcome::Changed
        }
        ct_event!(keycode press SHIFT-F(6)) => {
            state.inner_border_type = match state.inner_border_type {
                None => Some(BorderType::QuadrantOutside),
                Some(BorderType::Plain) => None,
                Some(BorderType::Double) => Some(BorderType::Plain),
                Some(BorderType::Rounded) => Some(BorderType::Double),
                Some(BorderType::Thick) => Some(BorderType::Rounded),
                Some(BorderType::QuadrantInside) => Some(BorderType::Thick),
                Some(BorderType::QuadrantOutside) => Some(BorderType::QuadrantInside),
            };
            Outcome::Changed
        }
        ct_event!(keycode press F(7)) => {
            state.resize = match state.resize {
                SplitResize::Neighbours => SplitResize::Full,
                SplitResize::Full => SplitResize::Neighbours,
            };
            Outcome::Changed
        }
        ct_event!(keycode press F(12)) => {
            if state.split.is_focused() {
                state.split.focus.set(false);
            } else {
                state.split.focus.set(true);
            }
            Outcome::Changed
        }
        _ => Outcome::Continue,
    });

    try_flow!(state.split.handle(event, Regular));
    try_flow!(match state.left.handle(event, Regular) {
        Outcome::Changed => Outcome::Changed,
        r => r,
    });
    try_flow!(state.right.handle(event, Regular));
    try_flow!(state.right_right.handle(event, Regular));
    try_flow!(match state.menu.handle(event, Regular) {
        MenuOutcome::Activated(0) => {
            istate.quit = true;
            Outcome::Changed
        }
        _ => Outcome::Continue,
    });

    Ok(Outcome::Continue)
}

static TEXT: &str="Stanislaus Kostka

Stanisław Kostka, S.J. (28 October 1550 – 15 August 1568) was a Polish novice in the Society of Jesus.

He was born at Rostkowo, Przasnysz County, Poland, on 28 October 1550, and died in Rome during the night of 14–15 August 1568. He is said to have foretold his death a few days before it occurred. He was canonized in 1726.

Biography

Family

His father was a senator of the Kingdom of Poland and castellan of Zakroczym;[1] his mother was Małgorzata Kryska from Drobni (Margaret de Drobniy Kryska), the sister and niece of the voivodes of Masovia and the aunt of the celebrated Chancellor of Poland, Feliks Kryski (Felix Kryski)(Szczęsny Kryski). He was the second of seven children. His older brother Paweł (Paul) survived to be present at the beatification ceremony of Stanislaus in 1605. At home, the two brothers were taught with firmness, even severity; its results were their piety, modesty, and temperance.

School life

On 25 July 1564, they arrived at Vienna with their tutor to attend the Jesuit college that had been opened four years before. Stanislaus was soon conspicuous among his classmates during his three years of schooling, not only for his amiability and cheerfulness of expression, but also for his growing religious fervour and piety.[2] His brother Paul said during the process of beatification that 'He devoted himself so completely to spiritual things that he frequently became unconscious, especially in the church of the Jesuit Fathers at Vienna. One of the practices of devotion which he joined while at Vienna was the Congregation of St. Barbara and Our Lady, 'of which he, with numbers of the pupils of the Society of Jesus' also belonged.[3] Stanislaus alleged to a fellow-member of the Society at Rome that Saint Barbara brought two angels to him during the course of a serious illness, in order to give him the Eucharist. His tutor, John Bilinski, witnessed the miracle, and though he himself did not see what Stanislaus claimed to see, he was certain that Stanislaus 'was not at all out of his mind through the violence of his sickness.'

Exasperated by his younger brother's piety, Paul began to mistreat Stanislaus. Stanislaus suffered the unjust treatment with remarkable stoicism and patience, but one night after Stanislaus had again suffered the harsh comments and blows of his brother, he turned on Paul with the words: 'Your rough treatment will end in my going away never to return, and you will have to explain my leaving to our father and mother.' Paul's sole reply was to swear violently at him.


Entry into the Society of Jesus

The thought of joining the Society of Jesus had already entered the mind of the saintly young man. It was six months, however, before he ventured to speak of this to the superiors of the Society. At Vienna they hesitated to receive him, fearing the tempest that would probably be raised by his father against the Society, which had just quieted a storm unleashed by other admissions to the Company. Another Jesuit suggested he go to Augsburg, Germany where Peter Canisius was provincial. The distance was over four hundred miles, which had to be made on foot, without equipment or guide or any other resources but that did not deter him.[1]
Stanislaus Kostka beaten by his brother, painting by Andrea Pozzo

    On the morning of the day on which he was to carry out his project he called his servant to him early and told him to notify his brother Paul and his tutor in the course of the morning that he would not be back that day to dinner. Then he started, exchanging the dress of gentleman for that of a mendicant, which was the only way to escape the curiosity of those he met. By nightfall Paul and the tutor comprehended that Stanislaus had fled, as he had threatened. They were seized with a fierce anger, and as the day was ended the fugitive had gained a day over them. They started to follow him, but were not able to overtake him; either their exhausted horses refused to go further, or a wheel of their carriage would break, or, as the tutor frankly declared, they had mistaken the route, having left the city by a different road from the one which Stanislaus had taken. It is noticeable that in his testimony Paul gives no explanation of his ill-luck.

    Stanislaus stayed for a month at Dillingen, where the provincial of that time, Saint Peter Canisius, put the young aspirant's vocation to the test by employing him in the boarding-school. He arrived 25 October 1567 in Rome. As he was greatly exhausted by the journey, the general of the order, Saint Francis Borgia, would not permit him to enter the novitiate of Saint Andrew until several days later. During the ten remaining months of his life, according to the testimony of the master of novices, Father Giulio Fazio, he was a model and mirror of religious perfection. Notwithstanding his very delicate constitution he did not spare himself the slightest penance.[4] He had such a burning fever in his chest that he was often obliged to apply cold compresses.[2]

Death

On the evening of the feast of Saint Lawrence (10 August), Stanislaus fell ill with a high fever, and clearly saw that his last hour had come. He wrote a letter to the Blessed Virgin begging her to call him to the skies there to celebrate with her the glorious anniversary of her Assumption (15 August).[5] His confidence in the Blessed Virgin, which had already brought him many favours, was this time again rewarded; on 15 August 1568, towards 4:00 in the morning, while he prayed he died. Many in the city proclaimed him a saint and people hastened from all parts to venerate his remains and to obtain, if possible, some relics.[6]

Sainthood

The Holy See ratified his beatification in 1605; he was canonized in 1726. St. Stanislaus is a popular saint of Poland, and many religious institutions have chosen him as the protector of their novitiates. The representations of him in art are quite varied; he is sometimes depicted receiving Holy Communion from the hands of angels, or receiving the Infant Jesus from the hands of the Virgin, or in the midst of a battle putting to flight the enemies of his country. At times he is depicted near a fountain putting a wet linen cloth on his breast. He is invoked for palpitations of the heart and for dangerous cases of illness.

On 15 August 2018 Pope Francis wrote to the Bishop of Płock in honor of the 450th anniversary of Stanislaus's death. In his message, Pope Francis cites a maxim of Stanislaus's: “Ad maiora natus sum – 'I was born for greater things'.[7]

Feast Day

    15 August – commemoration of death anniversary,[8]
    18 September – commemoration in Poland,[9]
    13 November – main commemoration,[10]

Depiction in art

Scipione Delfine portrait

There is a portrait by Scipione Delfine, the oldest of St. Stanislaus in existence. Having probably been painted at Rome within two years of his death, it may be regarded as the best likeness.

Additionally, Pierre Le Gros, the younger completed a marble statue of the saint in 1705.[11]
St. Stanisław Kostka on his death bed by Pierre Le Gros the Younger (1666–1719). Jesuit convent near Sant'Andrea al Quirinale, Rome.
Depiction in literature
Portrait in stained glass, Church Liesing

St. Thérèse of Lisieux wrote a “play” or “pious recreation” about Saint Stanislaus Kostka. Her sister Pauline had asked Thérèse to write verses and theatrical entertainment for liturgical and community festivals. Like Stanislaus, [Therese] thought she was going to die young having accomplished nothing, 'with empty hands.' From then on, her desire to do good after her death intensified.[12]

Dedications

The following are some places dedicated to him:

    Saint Stanislaus College in Georgetown, Guyana is the third highest ranking high school in the country and was founded in the name of the patron Saint Stanislaus Kostka in 1866.

    San Stanislao, a church in Palermo, Sicily.[13]

    Saint Stanislaus College in Bay Saint Louis, Mississippi, a Catholic day and residential school for boys in grades 7 to 12, founded in 1854 and chartered in 1870 as Saint Stanislaus College

    The former novitiate of the Central and Southern Province of the Society of Jesus –Saint Stanislaus Kostka at St.Charles College – located in Grand Coteau, Louisiana.[14] Saint Stanislaus is also a co-patron saint (along with Saint Ignatius of Loyola, founder of the Society of Jesus) of Strake Jesuit College Preparatory in Houston, Texas, where a statue of his image was erected in front of the Parsley Center, which houses an auditorium and music facilities. St. John's Jesuit High School and Academy of Toledo, Ohio, was one of the schools that used his name in their former house system.

    St. Stanislaus Kostka Church is a parish in Pittsburgh, Pennsylvania, US.

    Église Saint-Stanislas-de-Kostka] is a parish in Montreal located at 1350 Boulevard Saint-Joseph Est.[15]

    Saint-Stanislas-de-Kostka, Quebec, Canada, a municipality southwest of Montreal.

    St. Stanislaus Forane Church, Mala situated at Mala, Kerala, is reputed to be the only parish in India having St. Stanislaus as the patron saint.[citation needed]

    St. Stanislaus High School in Bandra, Mumbai, is a Jesuit founded in 1863.[16]

    The St. Stanislaus Institute in Ljubljana, Slovenia, an educational institution founded in 1901, is named for Stanislaus Kostka.[17]
    St. Stanislaus Kostka. St. Joseph's Church, Macao.

    Saint Kostka is also the patron saint of the Ateneo de Manila High School. At the Ateneo de Davao University, the grade school chapel, adorned with stained glass depictions of the life of Jesus Christ, was named after him.[citation needed]

    Saint Stanislaus Kostka is the patron saint of the attendees of Oblates of Saint Joseph Minor Seminary (a high school-level seminary in San Jose, Batangas, Philippines), as well as Manukan, Zamboanga del Norte Philippines.[citation needed]

    One of the junior campuses of the Jesuit school Xavier College in Melbourne, Australia, is named Kostka Hall; Boston College also has named one of their freshman dorms Kostka Hall. He is also a patron of the minor seminarians of the Oblates of St. Joseph Minor Seminary.[citation needed]

    Beaumont College, formerly a public school in Berkshire, was dedicated to Saint Stanislaus. In Belgium, there is Collège Saint-Stanislas in Mons.

    Saint Stanisław Kostka Church in Coventry, England is a church of the Polish Catholic Mission. https://www.parafia-coventry.org.uk/

    Saint Stanislaw Kostka is also the patron saint of The Polish Saturday School in Manchester, Lancashire,[1] which was founded in 1949 by the Manchester Polish Ex-Combatants Association and which supports the school to this day.[citation needed] The Polish name of the school is Polska Szkoła Przedmiotów Ojczystych im. św. Stanisława Kostki w Manchesterze.

    Saint Stanislaus High and Junior School in Gdynia is an independent private school located in Gdynia, Poland.[18]

    A school in Quezon City Philippines has a name of Kostka School of Quezon City which is named after St. Stanislaus Kostka.[citation needed]

    Also, a public school in Guyana is named after him, St. Stanislaus College.

    St. Stanislaus Kostka Church (Chicago) opened in 1867. By 1897, St. Stanislaus Kostka Parish was the largest parish in the United States with 8,000 families, totaling 40,000 people. There were twelve Masses each Sunday: six Masses in the upper church and another six Masses in the lower church. St. Stanislaus Kostka Parish is considered the mother church of the many Polish parishes.

    On November 10, 2011, the Church of Saint Stanislaus Kostka in Winona, Minnesota, was elevated by Pope Benedict XVI to the status of Minor Basilica, making it the first ever Basilica of Saint Stanislaus Kostka.

    The Brooklyn Diocese has two parishes and schools named after St. Stanislaus Kostka – one in Maspeth, Queens, and one in Greenpoint, Brooklyn.[citation needed]

    There is also a St. Stanislaus Kostka Roman Catholic Church in Staten Island, NY.[19]

    St. Stanislaus Church in Winchester, New Hampshire, US.[citation needed]

    St Stanislaus' College, in the rural city of Bathurst, New South Wales Australia.

    The Roman Catholic Diocese of Burlington has St. Stanislaus Kostka Church in West Rutland, Vermont. The Parish served the Polish families and the first Mass was celebrated on Easter Sunday 1906.[20]

    Saint Stanislaus Roman Catholic Church, located at 51 Lansdale Avenue in Lansdale Pennsylvania United States of America; founded in 1876 as a parish of the Archdiocese of Philadelphia

    The first Catholic secondary school in Tonga, founded in 1865, was named after St Stanislaus. This same school became known as Apifo'ou College since 1987.

    There is also a St. Stanislaus Kostka Church in Hamilton, Ontario Canada. It is located at 8 St. Ann Street. The 100th anniversary was celebrated in 2012 (1912–2012). http://stankostka.ca/

";
