#![allow(dead_code)]

use crate::adapter::paragraph::{ParagraphS, ParagraphSState};
use crate::mini_salsa::theme::THEME;
use crate::mini_salsa::{run_ui, setup_logging, MiniSalsaState};
use rat_event::{try_flow, HandleEvent, MouseOnly, Outcome};
use rat_scrolled::Scroll;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::widgets::{Block, StatefulWidget, Wrap};
use ratatui::Frame;

mod adapter;
mod mini_salsa;

fn main() -> Result<(), anyhow::Error> {
    setup_logging()?;

    let mut data = Data {
        sample1: TEXT_1.to_string(),
        sample2: TEXT_2.to_string(),
    };

    let mut state = State {
        text1: Default::default(),
        text2: Default::default(),
    };

    run_ui(
        "sparagraph",
        |_| {},
        handle_text,
        repaint_text,
        &mut data,
        &mut state,
    )
}

struct Data {
    pub(crate) sample1: String,
    pub(crate) sample2: String,
}

struct State {
    pub(crate) text1: ParagraphSState,
    pub(crate) text2: ParagraphSState,
}

fn repaint_text(
    frame: &mut Frame<'_>,
    area: Rect,
    data: &mut Data,
    _istate: &mut MiniSalsaState,
    state: &mut State,
) -> Result<(), anyhow::Error> {
    let l = Layout::horizontal([Constraint::Fill(1), Constraint::Fill(1)])
        .spacing(1)
        .split(area);

    ParagraphS::new(data.sample1.clone())
        .scroll(Scroll::new().overscroll_by(5).style(THEME.block()))
        .block(Block::bordered().style(THEME.block()))
        .style(THEME.text_input())
        .render(l[0], frame.buffer_mut(), &mut state.text1);

    ParagraphS::new(data.sample2.clone())
        .wrap(Wrap { trim: true })
        .scroll(Scroll::new().style(THEME.block()))
        .block(Block::bordered().style(THEME.block()))
        .style(THEME.text_input())
        .render(l[1], frame.buffer_mut(), &mut state.text2);

    Ok(())
}

fn handle_text(
    event: &crossterm::event::Event,
    _data: &mut Data,
    _istate: &mut MiniSalsaState,
    state: &mut State,
) -> Result<Outcome, anyhow::Error> {
    try_flow!(state.text1.handle(event, MouseOnly));
    try_flow!(state.text2.handle(event, MouseOnly));
    Ok(Outcome::Continue)
}

static TEXT_1: &str =
    "Fumarole minerals (or fumarolic minerals) are minerals which are deposited by fumarole
 exhalations. They form when gases and compounds desublimate or precipitate out of condensates,
 forming mineral deposits. They are mostly associated with volcanoes (as volcanic sublimate or
 fumarolic sublimate) following deposition from volcanic gas during an eruption or discharge from
 a volcanic vent or fumarole,[1] but have been encountered on burning coal deposits as well. They
 can be black or multicoloured and are often unstable upon exposure to the atmosphere.

Native sulfur, in this context called brimstone, is a common sublimate mineral and various halides,
 sulfides and sulfates occur in this environment associated with fumaroles and eruptions. A number
 of rare minerals are fumarole minerals, and at least 240 such minerals are known from Tolbachik
 volcano in Kamchatka, Russia. Other volcanoes where particular fumarole minerals have been
 discovered are Vulcano in Italy and Bezymyanny also in Russia.

Origin and appearance
Fumarole minerals in SEM images, from Mutnovsky volcano in Kamchatka. An electron
 micrograph, colour enhanced by optical microscopy, depicted width: 700 microns.

In fumaroles, minerals either form through desublimation from fumarole gases or through
 interactions of fumarole gases with country rock.[2] The former are known as sublimates and the
 latter as incrustations.[3] Some such deposits may also form through the interaction between liquid
 fumarole condensates and country rock[4] and are not always formed by desublimation.[5]
 Repeated cycles of primary deposition and secondary alteration may occur.[6] Volcanic gases such
 as hydrogen chloride, hydrogen fluoride, sulfur dioxide and water[7] can transport large amounts of
 elements, thus contributing to geochemical cycles on the surface and the formation of ore deposits
 at depth.[8] When these exhalations reach the atmosphere and cool, the minerals contained in them
 tend to precipitate out.[9]";

static TEXT_2: &str = "Craters of the Moon National Monument and Preserve is a U.S. national monument and national preserve in the Snake River Plain in central Idaho. It is along US 20 (concurrent with US 93 and US 26), between the small towns of Arco and Carey, at an average elevation of 5,900 feet (1,800 m) above sea level.

The Monument was established on May 2, 1924.[3] In November 2000, a presidential proclamation by President Clinton greatly expanded the Monument area. The 410,000-acre National Park Service portions of the expanded Monument were designated as Craters of the Moon National Preserve in August 2002.[1] It spreads across Blaine, Butte, Lincoln, Minidoka, and Power counties. The area is managed cooperatively by the National Park Service and the Bureau of Land Management (BLM).[4]

The Monument and Preserve encompass three major lava fields and about 400 square miles (1,000 km2) of sagebrush steppe grasslands to cover a total area of 1,117 square miles (2,893 km2). The Monument alone covers 343,000 acres (139,000 ha).[5] All three lava fields lie along the Great Rift of Idaho, with some of the best examples of open rift cracks in the world, including the deepest known on Earth at 800 feet (240 m). There are excellent examples of almost every variety of basaltic lava, as well as tree molds (cavities left by lava-incinerated trees), lava tubes (a type of cave), and many other volcanic features.[6]
Geography and geologic setting
Craters of the Moon within Idaho

Craters of the Moon is in south-central Idaho, midway between Boise and Yellowstone National Park. The lava field reaches southeastward from the Pioneer Mountains. Combined U.S. Highway 20–26–93 cuts through the northwestern part of the monument and provides access to it. However, the rugged landscape of the monument itself remains remote and undeveloped, with only one paved road across the northern end.

The Craters of the Moon Lava Field spreads across 618 square miles (1,601 km2) and is the largest mostly Holocene-aged basaltic lava field in the contiguous United States.[7] The Monument and Preserve contain more than 25 volcanic cones, including outstanding examples of spatter cones.[8] The 60 distinct solidified lava flows that form the Craters of the Moon Lava Field range in age from 15,000 to just 2,000 years.[9] The Kings Bowl and Wapi lava fields, both about 2,200 years old, are part of the National Preserve.

This lava field is the largest of several large beds of lava that erupted from the 53-mile (85 km) south-east to north-west trending Great Rift volcanic zone,[10] a line of weakness in the Earth's crust. Together with fields from other fissures they make up the Lava Beds of Idaho, which in turn are in the much larger Snake River Plain volcanic province. The Great Rift extends across almost the entire Snake River Plain.

Elevation at the visitor center is 5,910 feet (1,800 m) above sea level.[11]

Total average precipitation in the Craters of the Moon area is between 15–20 inches (380–510 mm) per year.[a][12] Most of this is lost in cracks in the basalt, only to emerge later in springs and seeps in the walls of the Snake River Canyon. Older lava fields on the plain support drought-resistant plants such as sagebrush, while younger fields, such as Craters of the Moon, only have a seasonal and very sparse cover of vegetation. When viewed from a distance, this cover disappears almost entirely, giving an impression of utter black desolation. Repeated lava flows over the last 15,000 years have raised the land surface enough to expose it to the prevailing southwesterly winds, which help to keep the area dry.[13] Together these conditions make life on the lava field difficult. "
 ;
