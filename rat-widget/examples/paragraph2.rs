#![allow(dead_code)]

use crate::mini_salsa::{MiniSalsaState, layout_grid, run_ui, setup_logging};
use rat_event::{HandleEvent, Outcome, Regular, ct_event, try_flow};
use rat_focus::{Focus, FocusBuilder, FocusFlag, Navigation};
use rat_scrolled::{Scroll, ScrollbarPolicy};
use rat_text::HasScreenCursor;
use rat_text::line_number::{LineNumberState, LineNumbers};
use rat_text::text_area::{TextArea, TextAreaState, TextWrap};
use rat_widget::paragraph::{Paragraph, ParagraphState};
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::widgets::{Block, StatefulWidget, Wrap};

mod mini_salsa;

fn main() -> Result<(), anyhow::Error> {
    setup_logging()?;

    let mut data = Data {
        sample: SAMPLE1.to_string(),
    };

    let mut state = State {
        sample: 0,
        wrap: true,
        line_numbers: Default::default(),
        para: Default::default(),
        line_numbers2: Default::default(),
        text: Default::default(),
    };
    state.text.set_text(SAMPLE1);

    run_ui(
        "paragraph1",
        |_, _, _| {},
        handle_text,
        repaint_text,
        &mut data,
        &mut state,
    )
}

struct Data {
    pub(crate) sample: String,
}

struct State {
    sample: u32,
    wrap: bool,
    line_numbers: LineNumberState,
    para: ParagraphState,
    line_numbers2: LineNumberState,
    text: TextAreaState,
}

fn repaint_text(
    frame: &mut Frame<'_>,
    area: Rect,
    data: &mut Data,
    istate: &mut MiniSalsaState,
    state: &mut State,
) -> Result<(), anyhow::Error> {
    let l0 = layout_grid::<6, 4>(
        area,
        Layout::horizontal([
            Constraint::Length(15),
            Constraint::Length(4),
            Constraint::Fill(1),
            Constraint::Length(4),
            Constraint::Fill(1),
            Constraint::Length(15),
        ]),
        Layout::vertical([
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ]),
    );

    let lln = Rect::new(
        l0[1][1].x,
        l0[1][1].y + 1,
        l0[1][1].width,
        l0[1][1].height - 2,
    );
    LineNumbers::new()
        .start(state.para.vscroll.offset as u32)
        .render(lln, frame.buffer_mut(), &mut state.line_numbers);

    let mut para = Paragraph::new(data.sample.clone())
        .vscroll(Scroll::new().policy(ScrollbarPolicy::Collapse))
        .hscroll(Scroll::new().policy(ScrollbarPolicy::Collapse))
        .block(
            Block::bordered()
                .title("Excerpt")
                .border_style(istate.theme.container_border())
                .title_style(istate.theme.container_border()),
        )
        .styles(istate.theme.paragraph_style());
    if state.wrap {
        para = para.wrap(Wrap::default());
    }
    para.render(l0[2][1], frame.buffer_mut(), &mut state.para);

    let lln = Rect::new(
        l0[3][1].x,
        l0[3][1].y + 1,
        l0[3][1].width,
        l0[3][1].height - 2,
    );
    LineNumbers::new().with_textarea(&state.text).render(
        lln,
        frame.buffer_mut(),
        &mut state.line_numbers2,
    );

    let mut text = TextArea::new()
        .styles(istate.theme.textview_style())
        .vscroll(Scroll::new().policy(ScrollbarPolicy::Collapse))
        .block(
            Block::bordered()
                .title("Excerpt")
                .border_style(istate.theme.container_border())
                .title_style(istate.theme.container_border()),
        );
    if state.wrap {
        text = text.text_wrap(TextWrap::Word(0));
    } else {
        text = text
            .text_wrap(TextWrap::Shift)
            .hscroll(Scroll::new().policy(ScrollbarPolicy::Collapse));
    }
    text.render(l0[4][1], frame.buffer_mut(), &mut state.text);

    if let Some(c) = state.text.screen_cursor() {
        frame.set_cursor_position(c);
    }

    Ok(())
}

fn focus(state: &State) -> Focus {
    let mut fb = FocusBuilder::default();
    fb.widget(&state.para);
    fb.widget_navigate(&state.text, Navigation::Regular);
    fb.build()
}

fn handle_text(
    event: &crossterm::event::Event,
    data: &mut Data,
    istate: &mut MiniSalsaState,
    state: &mut State,
) -> Result<Outcome, anyhow::Error> {
    istate.focus_outcome = focus(state).handle(event, Regular);

    try_flow!(state.para.handle(event, Regular));
    try_flow!(state.text.handle(event, Regular));

    try_flow!(match event {
        ct_event!(keycode press F(2)) => {
            state.wrap = !state.wrap;
            Outcome::Changed
        }
        ct_event!(keycode press F(3)) => {
            state.sample += 1;
            if state.sample > 1 {
                state.sample = 0;
            }
            match state.sample {
                0 => {
                    data.sample = SAMPLE1.to_string();
                    state.text.set_text(SAMPLE1);
                }
                1 => {
                    data.sample = SAMPLE2.to_string();
                    state.text.set_text(SAMPLE2);
                }
                _ => {}
            }
            state.para.set_line_offset(0);
            Outcome::Changed
        }
        _ => Outcome::Continue,
    });

    Ok(Outcome::Continue)
}

static SAMPLE1: &str = "Craters of the Moon National Monument and Preserve is a U.S. national monument and national preserve in the Snake River Plain in central Idaho. It is along US 20 (concurrent with US 93 and US 26), between the small towns of Arco and Carey, at an average elevation of 5,900 feet (1,800 m) above sea level.

The Monument was established on May 2, 1924.[3] In November 2000, a presidential proclamation by President Clinton greatly expanded the Monument area. The 410,000-acre National Park Service portions of the expanded Monument were designated as Craters of the Moon National Preserve in August 2002.[1] It spreads across Blaine, Butte, Lincoln, Minidoka, and Power counties. The area is managed cooperatively by the National Park Service and the Bureau of Land Management (BLM).[4]

The Monument and Preserve encompass three major lava fields and about 400 square miles (1,000 km2) of sagebrush steppe grasslands to cover a total area of 1,117 square miles (2,893 km2). The Monument alone covers 343,000 acres (139,000 ha).[5] All three lava fields lie along the Great Rift of Idaho, with some of the best examples of open rift cracks in the world, including the deepest known on Earth at 800 feet (240 m). There are excellent examples of almost every variety of basaltic lava, as well as tree molds (cavities left by lava-incinerated trees), lava tubes (a type of cave), and many other volcanic features.[6]
Geography and geologic setting
Craters of the Moon within Idaho

Craters of the Moon is in south-central Idaho, midway between Boise and Yellowstone National Park. The lava field reaches southeastward from the Pioneer Mountains. Combined U.S. Highway 20–26–93 cuts through the northwestern part of the monument and provides access to it. However, the rugged landscape of the monument itself remains remote and undeveloped, with only one paved road across the northern end.

The Craters of the Moon Lava Field spreads across 618 square miles (1,601 km2) and is the largest mostly Holocene-aged basaltic lava field in the contiguous United States.[7] The Monument and Preserve contain more than 25 volcanic cones, including outstanding examples of spatter cones.[8] The 60 distinct solidified lava flows that form the Craters of the Moon Lava Field range in age from 15,000 to just 2,000 years.[9] The Kings Bowl and Wapi lava fields, both about 2,200 years old, are part of the National Preserve.

This lava field is the largest of several large beds of lava that erupted from the 53-mile (85 km) south-east to north-west trending Great Rift volcanic zone,[10] a line of weakness in the Earth's crust. Together with fields from other fissures they make up the Lava Beds of Idaho, which in turn are in the much larger Snake River Plain volcanic province. The Great Rift extends across almost the entire Snake River Plain.

Elevation at the visitor center is 5,910 feet (1,800 m) above sea level.[11]

Total average precipitation in the Craters of the Moon area is between 15–20 inches (380–510 mm) per year.[a][12] Most of this is lost in cracks in the basalt, only to emerge later in springs and seeps in the walls of the Snake River Canyon. Older lava fields on the plain support drought-resistant plants such as sagebrush, while younger fields, such as Craters of the Moon, only have a seasonal and very sparse cover of vegetation. When viewed from a distance, this cover disappears almost entirely, giving an impression of utter black desolation. Repeated lava flows over the last 15,000 years have raised the land surface enough to expose it to the prevailing southwesterly winds, which help to keep the area dry.[13] Together these conditions make life on the lava field difficult. ";

static SAMPLE2: &str = r##"Constantine I

(Latin: Flavius Valerius Constantinus; 27 February c. 272 – 22 May 337), also known as Constantine the Great, was a Roman emperor from AD 306 to 337 and the first Roman emperor to convert to Christianity. He played a pivotal role in elevating the status of Christianity in Rome, decriminalizing Christian practice and ceasing Christian persecution in a period referred to as the Constantinian shift. This initiated the Christianization of the Roman Empire. Constantine is associated with the religiopolitical ideology known as Caesaropapism, which epitomizes the unity of church and state. He founded the city of Constantinople and made it the capital of the Empire, which remained so for over a millennium.
Born in Naissus, in Dardania within Moesia Superior (now Niš, Serbia), Constantine was the son of Flavius Constantius, a Roman army officer of Illyrian origin who had been one of the four rulers of the Tetrarchy. His mother, Helena, was a Greek woman of low birth, probably from Asia Minor in modern Turkey. Later canonised as a saint, she is traditionally credited for the conversion of her son. Constantine began his career under emperors Diocletian and Galerius in the eastern provinces. He fought the Persians before being recalled west in 305 to campaign alongside his father in the province of Britannia. After his father's death in 306, Constantine was proclaimed as augustus (emperor) by his army at Eboracum (York, England). He eventually emerged victorious in the civil wars against emperors Maxentius and Licinius to become the sole ruler of the Roman Empire by 324.
Upon his ascension, Constantine enacted numerous reforms to strengthen the empire. He restructured the government, separating civil and military authorities. To combat inflation, he introduced the solidus, a new gold coin that became the standard for Byzantine and European currencies for more than a thousand years. The Roman army was reorganised to consist of mobile units (comitatenses), often around the Emperor, to serve on campaigns against external enemies or Roman rebels, and frontier-garrison troops (limitanei) which were capable of countering barbarian raids, but less and less capable, over time, of countering full-scale barbarian invasions. Constantine pursued successful campaigns against the tribes on the Roman frontiers—such as the Franks, the Alemanni, the Goths, and the Sarmatians—and resettled territories abandoned by his predecessors during the Crisis of the Third Century with citizens of Roman culture.
Although Constantine lived much of his life as a pagan and later as a catechumen, he began to favour Christianity beginning in 312, finally becoming a Christian and being baptised by Eusebius of Nicomedia, an Arian bishop, although the Catholic Church and the Coptic Orthodox Church maintain that he was baptised by Pope Sylvester I. He played an influential role in the proclamation of the Edict of Milan in 313, which declared tolerance for Christianity in the Roman Empire. He convoked the First Council of Nicaea in 325 which produced the statement of Christian belief known as the Nicene Creed. The Church of the Holy Sepulchre was built on his orders at the claimed site of Jesus' tomb in Jerusalem and was deemed the holiest place in all of Christendom. The papal claim to temporal power in the High Middle Ages was based on the fabricated Donation of Constantine. He has historically been referred to as the "First Christian Emperor", but while he did favour the Christian Church, some modern scholars debate his beliefs and even his comprehension of Christianity. Nevertheless, he is venerated as a saint in Eastern Christianity, and he did much to push Christianity towards the mainstream of Roman culture.
The age of Constantine marked a distinct epoch in the history of the Roman Empire and a pivotal moment in the transition from classical antiquity to the Middle Ages. He built a new imperial residence in the city of Byzantium and renamed it New Rome, later adopting the name Constantinople after himself, where it was located in modern Istanbul. It subsequently became the capital of the empire for more than a thousand years, the later Eastern Roman Empire often being referred to in English as the Byzantine Empire, a term never used by the Empire, invented by German historian Hieronymus Wolf. His more immediate political legacy was that he replaced Diocletian's Tetrarchy with the de facto principle of dynastic succession by leaving the empire to his sons and other members of the Constantinian dynasty. His reputation flourished during the lifetime of his children and for centuries after his reign. The medieval church held him up as a paragon of virtue, while secular rulers invoked him as a prototype, a point of reference, and the symbol of imperial legitimacy and identity. At the beginning of the Renaissance, there were more critical appraisals of his reign with the rediscovery of anti-Constantinian sources. Trends in modern and recent scholarship have attempted to balance the extremes of previous scholarship.


Sources
Constantine was a ruler of major importance and has always been a controversial figure. The fluctuations in his reputation reflect the nature of the ancient sources for his reign. These are abundant and detailed, but they have been strongly influenced by the official propaganda of the period and are often one-sided; no contemporaneous histories or biographies dealing with his life and rule have survived. The nearest replacement is Eusebius's Vita Constantini—a mixture of eulogy and hagiography written between 335 and circa 339—that extols Constantine's moral and religious virtues. The Vita creates a contentiously positive image of Constantine, and modern historians have frequently challenged its reliability. The fullest secular life of Constantine is the anonymous Origo Constantini, a work of uncertain date which focuses on military and political events to the neglect of cultural and religious matters.
Lactantius' De mortibus persecutorum, a political Christian pamphlet on the reigns of Diocletian and the Tetrarchy, provides valuable but tendentious detail on Constantine's predecessors and early life. The ecclesiastical histories of Socrates, Sozomen, and Theodoret describe the ecclesiastic disputes of Constantine's later reign. Written during the reign of Theodosius II (r. 402–450), a century after Constantine's reign, these ecclesiastical historians obscure the events and theologies of the Constantinian period through misdirection, misrepresentation, and deliberate obscurity. The contemporary writings of the orthodox Christian Athanasius and the ecclesiastical history of the Arian Philostorgius also survive, though their biases are no less firm.
The epitomes of Aurelius Victor (De Caesaribus), Eutropius (Breviarium), Festus (Breviarium), and the anonymous author of the Epitome de Caesaribus offer compressed secular political and military histories of the period. Although not Christian, the epitomes paint a favourable image of Constantine but omit reference to Constantine's religious policies. The Panegyrici Latini, a collection of panegyrics from the late 3rd and early 4th centuries, provides valuable information on the politics and ideology of the tetrarchic period and the early life of Constantine. Contemporary architecture—such as the Arch of Constantine in Rome and palaces in Gamzigrad and Córdoba—epigraphic remains, and the coinage of the era complement the literary sources.

Early life

Constantine was born on 27 February, c. AD 272 in the city of Naissus, a time where the unity of the Empire was threatened by the breakaway wars of the Palmyrene Empire. The city—which is modern day Niš in Serbia—was located in Dardania within Moesia Superior. His father was Flavius Constantius an Illyrian who was born in the same region, and a native of the province of Moesia. His original full name, as well as that of his father, is not known. His praenomen is variously given as Lucius, Marcus and Gaius. Whatever the case, praenomina had already disappeared from most public records by this time. He also adopted the name "Valerius", the nomen of emperor Diocletian, following his father's ascension as caesar.
Constantine probably spent little time with his father who was an officer in the Roman army, part of Emperor Aurelian's imperial bodyguard. Being described as a tolerant and politically skilled man, Constantius advanced through the ranks, earning the governorship of Dalmatia from Emperor Diocletian, another of Aurelian's companions from Illyricum, in 284 or 285. Constantine's mother was Helena, a Greek woman of low social standing from Helenopolis of Bithynia. It is uncertain whether she was legally married to Constantius or merely his concubine. His main language was Latin, and during his public speeches he needed Greek translators.


In April 286, Diocletian declared Maximian, another colleague from Illyricum, his co-emperor. Each emperor would have his own court, his own military and administrative faculties, and each would rule with a separate praetorian prefect as chief lieutenant. Maximian ruled in the West, from his capitals at Mediolanum (Milan, Italy) or Augusta Treverorum (Trier, Germany), while Diocletian ruled in the East, from Nicomedia (İzmit, Turkey). The division was merely pragmatic: the empire was called "indivisible" in official panegyric, and both emperors could move freely throughout the empire. In 288, Maximian appointed Constantius to serve as his praetorian prefect in Gaul. Constantius left Helena to marry Maximian's stepdaughter Theodora in 288 or 289.
Diocletian divided the empire again in 293, appointing two caesars to rule over further subdivisions of East and West. Each would be subordinate to his respective augustus but would act with supreme authority in his assigned lands. This system would later be called the Tetrarchy. Diocletian's first appointee for the office of Caesar was Constantius; his second was Galerius, a native of Felix Romuliana. According to Lactantius, Galerius was a brutal, animalistic man. Although he shared the paganism of Rome's aristocracy, he seemed to them an alien figure, a semi-barbarian. On 1 March, Constantius was promoted to the office of Caesar, and dispatched to Gaul to fight the rebels Carausius and Allectus. In spite of meritocratic overtones, the Tetrarchy retained vestiges of hereditary privilege, and Constantine became the prime candidate for future appointment as Caesar as soon as his father took the position. Constantine went to the court of Diocletian, where he lived as his father's heir presumptive.

In the East
Constantine received a formal education at Diocletian's court, where he learned Latin literature, Greek, and philosophy. The cultural environment in Nicomedia was open, fluid, and socially mobile; in it, Constantine could mix with intellectuals both pagan and Christian. He may have attended the lectures of Lactantius, a Christian scholar of Latin in the city. Because Diocletian did not completely trust Constantius—none of the Tetrarchs fully trusted their colleagues—Constantine was held as something of a hostage, a tool to ensure Constantius' best behavior. Constantine was nonetheless a prominent member of the court: he fought for Diocletian and Galerius in Asia and served in a variety of tribunates; he campaigned against barbarians on the Danube in 296 and fought the Persians under Diocletian in Syria in 297, as well as under Galerius in Mesopotamia in 298–299. By late 305, he had become a tribune of the first order, a tribunus ordinis primi.


Constantine had returned to Nicomedia from the eastern front by the spring of 303, in time to witness the beginnings of Diocletian's "Great Persecution", the most severe persecution of Christians in Roman history. In late 302, Diocletian and Galerius sent a messenger to the oracle of Apollo at Didyma with an inquiry about Christians. Constantine could recall his presence at the palace when the messenger returned and Diocletian accepted the imperial court's demands for universal persecution. On 23 February 303, Diocletian ordered the destruction of Nicomedia's new church, condemned its scriptures to the flames, and had its treasures seized. In the months that followed, churches and scriptures were destroyed, Christians were deprived of official ranks, and priests were imprisoned. It is unlikely that Constantine played any role in the persecution. In his later writings, he attempted to present himself as an opponent of Diocletian's "sanguinary edicts" against the "Worshippers of God", but nothing indicates that he opposed it effectively at the time. Although no contemporary Christian challenged Constantine for his inaction during the persecutions, it remained a political liability throughout his life.
On 1 May 305, Diocletian, as a result of a debilitating sickness taken in the winter of 304–305, announced his resignation. In a parallel ceremony in Milan, Maximian did the same. Lactantius states that Galerius manipulated the weakened Diocletian into resigning and forced him to accept Galerius' allies in the imperial succession. According to Lactantius, the crowd listening to Diocletian's resignation speech believed, until the last moment, that Diocletian would choose Constantine and Maxentius (Maximian's son) as his successors. It was not to be: Constantius and Galerius were promoted to augusti, while Severus and Maximinus, Galerius' nephew, were appointed their caesars respectively. Constantine and Maxentius were ignored.
Some of the ancient sources detail plots that Galerius made on Constantine's life in the months following Diocletian's abdication. They assert that Galerius assigned Constantine to lead an advance unit in a cavalry charge through a swamp on the middle Danube, made him enter into single combat with a lion, and attempted to kill him in hunts and wars. Constantine always emerged victorious: the lion emerged from the contest in a poorer condition than Constantine; Constantine returned to Nicomedia from the Danube with a Sarmatian captive to drop at Galerius' feet. It is uncertain how much these tales can be trusted.

In the West
Constantine recognised the implicit danger in remaining at Galerius' court, where he was held as a virtual hostage. His career depended on being rescued by his father in the West. Constantius was quick to intervene. In the late spring or early summer of 305, Constantius requested leave for his son to help him campaign in Britain. After a long evening of drinking, Galerius granted the request. Constantine's later propaganda describes how he fled the court in the night, before Galerius could change his mind. He rode from post-house to post-house at high speed, hamstringing every horse in his wake. By the time Galerius awoke the following morning, Constantine had fled too far to be caught. Constantine joined his father in Gaul, at Bononia (Boulogne) before the summer of 305.


From Bononia, they crossed the English Channel to Britain and made their way to Eboracum (York), capital of the province of Britannia Secunda and home to a large military base. Constantine was able to spend a year in northern Britain at his father's side, campaigning against the Picts beyond Hadrian's Wall in the summer and autumn. Constantius' campaign, like that of Septimius Severus before it, probably advanced far into the north without achieving great success. Constantius had become severely sick over the course of his reign and died on 25 July 306 in Eboracum. Before dying, he declared his support for raising Constantine to the rank of full Augustus. The Alamannic king Chrocus, a barbarian taken into service under Constantius, then proclaimed Constantine as augustus. The troops loyal to Constantius' memory followed him in acclamation. Gaul and Britain quickly accepted his rule; Hispania, which had been in his father's domain for less than a year, rejected it.
Constantine sent Galerius an official notice of Constantius' death and his own acclamation. Along with the notice, he included a portrait of himself in the robes of an augustus. The portrait was wreathed in bay. He requested recognition as heir to his father's throne and passed off responsibility for his unlawful ascension on his army, claiming they had "forced it upon him". Galerius was put into a fury by the message; he almost set the portrait and messenger on fire. His advisers calmed him and argued that outright denial of Constantine's claims would mean certain war. Galerius was compelled to compromise: he granted Constantine the title "caesar" rather than "augustus" (the latter office went to Severus instead). Wishing to make it clear that he alone gave Constantine legitimacy, Galerius personally sent Constantine the emperor's traditional purple robes. Constantine accepted the decision, knowing that it would remove doubts as to his legitimacy.

Reign

Constantine's share of the empire consisted of Britain, Gaul, and Spain, and he commanded one of the largest Roman armies which was stationed along the important Rhine frontier. He remained in Britain after his promotion to emperor, driving back the tribes of the Picts and securing his control in the northwestern dioceses. He completed the reconstruction of military bases begun under his father's rule, and he ordered the repair of the region's roadways. He then left for Augusta Treverorum (Trier) in Gaul, the Tetrarchic capital of the northwestern Roman Empire. The Franks learned of Constantine's acclamation and invaded Gaul across the lower Rhine over the winter of 306–307. He drove them back beyond the Rhine and captured kings Ascaric and Merogais; the kings and their soldiers were fed to the beasts of Trier Amphitheater in the adventus (arrival) celebrations which followed.



Constantine began a major expansion of Trier. He strengthened the circuit wall around the city with military towers and fortified gates, and he began building a palace complex in the northeastern part of the city. To the south of his palace, he ordered the construction of a large formal audience hall and a massive imperial bathhouse. He sponsored many building projects throughout Gaul during his tenure as emperor of the West, especially in Augustodunum (Autun) and Arelate (Arles). According to Lactantius, Constantine followed a tolerant policy towards Christianity, although he was not yet a Christian. He probably judged it a more sensible policy than open persecution and a way to distinguish himself from the "great persecutor" Galerius. He decreed a formal end to persecution and returned to Christians all that they had lost during them.
Constantine was largely untried and had a hint of illegitimacy about him; he relied on his father's reputation in his early propaganda, which gave as much coverage to his father's deeds as to his. His military skill and building projects, however, soon gave the panegyrist the opportunity to comment favourably on the similarities between father and son, and Eusebius remarked that Constantine was a "renewal, as it were, in his own person, of his father's life and reign". Constantinian coinage, sculpture, and oratory also show a tendency for disdain towards the "barbarians" beyond the frontiers. He minted a coin issue after his victory over the Alemanni which depicts weeping and begging Alemannic tribesmen, "the Alemanni conquered" beneath the phrase "Romans' rejoicing". There was little sympathy for these enemies; as his panegyrist declared, "It is a stupid clemency that spares the conquered foe."

Maxentius' rebellion

Following Galerius' recognition of Constantine as caesar, Constantine's portrait was brought to Rome, as was customary. Maxentius mocked the portrait's subject as the son of a harlot and lamented his own powerlessness. Maxentius, envious of Constantine's authority, seized the title of emperor on 28 October 306. Galerius refused to recognize him but failed to unseat him. Severus was sent against Maxentius in April 307, but during the campaign, Severus' armies, previously under command of Maxentius' father Maximian, defected, and Severus was seized and imprisoned. Maximian, brought out of retirement by his son's rebellion, left for Gaul to confer with Constantine. He offered to marry his daughter Fausta to Constantine and elevate him to augustan rank. In return, Constantine would reaffirm the old family alliance between Maximian and Constantius and offer support to Maxentius' cause in Italy. Constantine accepted and married Fausta in Trier in summer 307. Constantine gave Maxentius his meagre support, offering Maxentius political recognition.
Constantine remained aloof from the Italian conflict, however. Over the spring and summer of 307, he had left Gaul for Britain to avoid any involvement in the Italian turmoil; now, instead of giving Maxentius military aid, he sent his troops against Germanic tribes along the Rhine. In 308, he raided the territory of the Bructeri and made a bridge across the Rhine at Colonia Agrippinensium (Cologne). In 310, he marched to the northern Rhine and fought the Franks. When not campaigning, he toured his lands advertising his benevolence and supporting the economy and the arts. His refusal to participate in the war increased his popularity among his people and strengthened his power base in the West. Maximian returned to Rome in the winter of 307–308 but soon fell out with his son. In early 308, after a failed attempt to usurp Maxentius' title, Maximian returned to Constantine's court.
On 11 November 308, Galerius called a general council at the military city of Carnuntum (Petronell-Carnuntum, Austria) to resolve the instability in the western provinces. In attendance were Diocletian, briefly returned from retirement, Galerius, and Maximian. Maximian was forced to abdicate again and Constantine was again demoted to caesar. Licinius, one of Galerius' old military companions, was appointed augustus in the western regions. The new system did not last long: Constantine refused to accept the demotion and continued to style himself as augustus on his coinage, even as other members of the Tetrarchy referred to him as a caesar on theirs. Maximinus was frustrated that he had been passed over for promotion while the newcomer Licinius had been raised to the office of augustus and demanded that Galerius promote him. Galerius offered to call both Maximinus and Constantine "sons of the augusti", but neither accepted the new title. By the spring of 310, Galerius was referring to both men as augusti.

Maximian's rebellion

In 310, a dispossessed Maximian rebelled against Constantine while Constantine was away campaigning against the Franks. Maximian had been sent south to Arles with a contingent of Constantine's army, in preparation for any attacks by Maxentius in southern Gaul. He announced that Constantine was dead and took up the imperial purple. In spite of a large donative pledge to any who would support him as emperor, most of Constantine's army remained loyal to their emperor, and Maximian was soon compelled to leave. When Constantine heard of the rebellion, he abandoned his campaign against the Franks and marched his army up the Rhine. At Cabillunum (Chalon-sur-Saône), he moved his troops onto waiting boats to row down the slow waters of the Saône to the quicker waters of the Rhone. He disembarked at Lugdunum (Lyon). Maximian fled to Massilia (Marseille), a town better able to withstand a long siege than Arles. It made little difference, however, as loyal citizens opened the rear gates to Constantine. Maximian was captured and reproved for his crimes. Constantine granted some clemency but strongly encouraged his suicide. In July 310, Maximian hanged himself.
In spite of the earlier rupture in their relations, Maxentius was eager to present himself as his father's devoted son after his death. He began minting coins with his father's deified image, proclaiming his desire to avenge Maximian's death. Constantine initially presented the suicide as an unfortunate family tragedy. By 311, however, he was spreading another version. According to this, after Constantine had pardoned him, Maximian planned to murder Constantine in his sleep. Fausta learned of the plot and warned Constantine, who put a eunuch in his own place in bed. Maximian was apprehended when he killed the eunuch and was offered suicide, which he accepted. Along with using propaganda, Constantine instituted a damnatio memoriae on Maximian, destroying all inscriptions referring to him and eliminating any public work bearing his image.
The death of Maximian required a shift in Constantine's public image. He could no longer rely on his connection to the elder Emperor Maximian and needed a new source of legitimacy. In a speech delivered in Gaul on 25 July 310, the anonymous orator reveals a previously unknown dynastic connection to Claudius II, a 3rd-century emperor famed for defeating the Goths and restoring order to the empire. Breaking away from tetrarchic models, the speech emphasizes Constantine's ancestral prerogative to rule, rather than principles of imperial equality. The new ideology expressed in the speech made Galerius and Maximian irrelevant to Constantine's right to rule. Indeed, the orator emphasizes ancestry to the exclusion of all other factors: "No chance agreement of men, nor some unexpected consequence of favour, made you emperor," the orator declares to Constantine.
The oration also moves away from the religious ideology of the Tetrarchy, with its focus on twin dynasties of Jupiter and Hercules. Instead, the orator proclaims that Constantine experienced a divine vision of Apollo and Victory granting him laurel wreaths of health and a long reign. In the likeness of Apollo, Constantine recognised himself as the saving figure to whom would be granted "rule of the whole world", as the poet Virgil had once foretold. The oration's religious shift is paralleled by a similar shift in Constantine's coinage. In his early reign, the coinage of Constantine advertised Mars as his patron. From 310 on, Mars was replaced by Sol Invictus, a god conventionally identified with Apollo. There is little reason to believe that either the dynastic connection or the divine vision are anything other than fiction, but their proclamation strengthened Constantine's claims to legitimacy and increased his popularity among the citizens of Gaul.

Civil wars

War against Maxentius

By the middle of 310, Galerius had become too ill to involve himself in imperial politics. His final act survives: a letter to provincials posted in Nicomedia on 30 April 311, proclaiming an end to the persecutions, and the resumption of religious toleration.
Eusebius maintains "divine providence [...] took action against the perpetrator of these crimes" and gives a graphic account of Galerius' demise:
"Without warning suppurative inflammation broke out round the middle of his genitals, then a deep-seated fistula ulcer; these ate their way incurably into his innermost bowels. From them came a teeming indescribable mass of worms, and a sickening smell was given off, for the whole of his hulking body, thanks to over eating, had been transformed even before his illness into a huge lump of flabby fat, which then decomposed and presented those who came near it with a revolting and horrifying sight."

Galerius died soon after the edict's proclamation, destroying what little remained of the Tetrarchy. Maximinus mobilised against Licinius and seized Asia Minor. A hasty peace was signed on a boat in the middle of the Bosphorus. While Constantine toured Britain and Gaul, Maxentius prepared for war. He fortified northern Italy and strengthened his support in the Christian community by allowing it to elect Eusebius as bishop of Rome.
Maxentius' rule was nevertheless insecure. His early support dissolved in the wake of heightened tax rates and depressed trade; riots broke out in Rome and Carthage; and Domitius Alexander was able to briefly usurp his authority in Africa. By 312, he was a man barely tolerated, not one actively supported, even among Christian Italians. In the summer of 311, Maxentius mobilised against Constantine while Licinius was occupied with affairs in the East. He declared war on Constantine, vowing to avenge his father's "murder". To prevent Maxentius from forming an alliance against him with Licinius, Constantine forged his own alliance with Licinius over the winter of 311–312 and offered him his sister Constantia in marriage. Maximinus considered Constantine's arrangement with Licinius an affront to his authority. In response, he sent ambassadors to Rome, offering political recognition to Maxentius in exchange for a military support, which Maxentius accepted. According to Eusebius, inter-regional travel became impossible, and there was military buildup everywhere. There was "not a place where people were not expecting the onset of hostilities every day".


Constantine's advisers and generals cautioned against preemptive attack on Maxentius; even his soothsayers recommended against it, stating that the sacrifices had produced unfavourable omens. Constantine, with a spirit that left a deep impression on his followers, inspiring some to believe that he had some form of supernatural guidance, ignored all these cautions. Early in the spring of 312, Constantine crossed the Cottian Alps with a quarter of his army, a force numbering about 40,000. The first town his army encountered was Segusium (Susa, Italy), a heavily fortified town that shut its gates to him. Constantine ordered his men to set fire to its gates and scale its walls. He took the town quickly. Constantine ordered his troops not to loot the town and advanced into northern Italy.
At the approach to the west of the important city of Augusta Taurinorum (Turin, Italy), Constantine met a large force of heavily armed Maxentian cavalry. In the ensuing Battle of Turin Constantine's army encircled Maxentius' cavalry, flanked them with his own cavalry, and dismounted them with blows from his soldiers' iron-tipped clubs. Constantine's armies emerged victorious. Turin refused to give refuge to Maxentius' retreating forces, opening its gates to Constantine instead. Other cities of the north Italian plain sent Constantine embassies of congratulation for his victory. He moved on to Milan, where he was met with open gates and jubilant rejoicing. Constantine rested his army in Milan until mid-summer 312, when he moved on to Brixia (Brescia).
Brescia's army was easily dispersed, and Constantine quickly advanced to Verona where a large Maxentian force was camped. Ruricius Pompeianus, general of the Veronese forces and Maxentius' praetorian prefect, was in a strong defensive position since the town was surrounded on three sides by the Adige. Constantine sent a small force north of the town in an attempt to cross the river unnoticed. Ruricius sent a large detachment to counter Constantine's expeditionary force but was defeated. Constantine's forces successfully surrounded the town and laid siege. Ruricius gave Constantine the slip and returned with a larger force to oppose Constantine. Constantine refused to let up on the siege and sent only a small force to oppose him. In the desperately fought encounter that followed, Ruricius was killed and his army destroyed. Verona surrendered soon afterwards, followed by Aquileia, Mutina (Modena), and Ravenna. The road to Rome was now wide open to Constantine.


Maxentius prepared for the same type of war he had waged against Severus and Galerius: he sat in Rome and prepared for a siege. He still controlled Rome's Praetorian Guard, was well-stocked with African grain, and was surrounded on all sides by the seemingly impregnable Aurelian Walls. He ordered all bridges across the Tiber cut, reportedly on the counsel of the gods, and left the rest of central Italy undefended; Constantine secured that region's support without challenge. Constantine progressed slowly along the Via Flaminia, allowing the weakness of Maxentius to draw his regime further into turmoil. Maxentius' support continued to weaken: at chariot races on 27 October, the crowd openly taunted Maxentius, shouting that Constantine was invincible. Maxentius, no longer certain that he would emerge from a siege victorious, built a temporary boat bridge across the Tiber in preparation for a field battle against Constantine. On 28 October 312, the sixth anniversary of his reign, he approached the keepers of the Sibylline Books for guidance. The keepers prophesied that, on that very day, "the enemy of the Romans" would die. Maxentius advanced north to meet Constantine in battle.

Constantine adopts the Greek letters Chi Rho for Christ's initials




Maxentius' forces were still twice the size of Constantine's, and he organised them in long lines facing the battle plain with their backs to the river. Constantine's army arrived on the field bearing unfamiliar symbols on their standards and their shields. According to Lactantius "Constantine was directed in a dream to cause the heavenly sign to be delineated on the shields of his soldiers, and so to proceed to battle. He did as he had been commanded, and he marked on their shields the letter Χ, with a perpendicular line drawn through it and turned round thus at the top, being the cipher of Christ. Having this sign (☧), his troops stood to arms." Eusebius describes a vision that Constantine had while marching at midday in which "he saw with his own eyes the trophy of a cross of light in the heavens, above the sun, and bearing the inscription, In Hoc Signo Vinces" ("In this sign thou shalt conquer"). In Eusebius's account, Constantine had a dream the following night in which Christ appeared with the same heavenly sign and told him to make an army standard in the form of the labarum. Eusebius is vague about when and where these events took place, but it enters his narrative before the war begins against Maxentius. He describes the sign as Chi (Χ) traversed by Rho (Ρ) to form ☧, representing the first two letters of the Greek word ΧΡΙΣΤΟΣ (Christos). A medallion was issued at Ticinum in 315 which shows Constantine wearing a helmet emblazoned with the Chi Rho, and coins issued at Siscia in 317/318 repeat the image. The figure was otherwise rare and is uncommon in imperial iconography and propaganda before the 320s. It was not completely unknown, however, being an abbreviation of the Greek word chrēston (good), having previously appeared on the coins of Ptolemy III Euergetes in the 3rd century BC. Following Constantine, centuries of Christians invoked the miraculous or the supernatural when justifying or describing their warfare.
Constantine deployed his own forces along the whole length of Maxentius' line. He ordered his cavalry to charge, and they broke Maxentius' cavalry. He then sent his infantry against Maxentius' infantry, pushing many into the Tiber where they were slaughtered and drowned. The battle was brief, and Maxentius' troops were broken before the first charge. His horse guards and praetorians initially held their position, but they broke under the force of a Constantinian cavalry charge; they also broke ranks and fled to the river. Maxentius rode with them and attempted to cross the bridge of boats (Ponte Milvio), but he was pushed into the Tiber and drowned by the mass of his fleeing soldiers.

In Rome

Constantine entered Rome on 29 October 312 and staged a grand adventus in the city which was met with jubilation. Maxentius' body was fished out of the Tiber and decapitated, and his head was paraded through the streets for all to see. After the ceremonies, the disembodied head was sent to Carthage, and Carthage offered no further resistance. Unlike his predecessors, Constantine neglected to make the trip to the Capitoline Hill and perform customary sacrifices at the Temple of Jupiter. However, he did visit the Senatorial Curia Julia, and he promised to restore its ancestral privileges and give it a secure role in his reformed government; there would be no revenge against Maxentius' supporters. In response, the Senate decreed him "title of the first name", which meant that his name would be listed first in all official documents, and they acclaimed him as "the greatest augustus". He issued decrees returning property that was lost under Maxentius, recalling political exiles, and releasing Maxentius' imprisoned opponents.
An extensive propaganda campaign followed, during which Maxentius' image was purged from all public places. He was written up as a "tyrant" and set against an idealised image of Constantine the "liberator". Eusebius is the best representative of this strand of Constantinian propaganda. Maxentius' rescripts were declared invalid, and the honours that he had granted to leaders of the Senate were also invalidated. Constantine also attempted to remove Maxentius' influence on Rome's urban landscape. All structures built by him were rededicated to Constantine, including the Temple of Romulus and the Basilica of Maxentius. At the focal point of the basilica, a stone statue was erected of Constantine holding the Christian labarum in its hand. Its inscription bore the message which the statue illustrated: "By this sign, Constantine had freed Rome from the yoke of the tyrant."
Constantine also sought to upstage Maxentius' achievements. For example, the Circus Maximus was redeveloped so that its seating capacity was 25 times larger than that of Maxentius' racing complex on the Via Appia. Maxentius' strongest military supporters were neutralised when he disbanded the Praetorian Guard and Imperial Horse Guard. The tombstones of the Imperial Horse Guard were ground up and used in a basilica on the Via Labicana, and their former base was redeveloped into the Lateran Basilica on 9 November 312—barely two weeks after Constantine captured the city. The Legio II Parthica was removed from Albano Laziale, and the remainder of Maxentius' armies were sent to do frontier duty on the Rhine.

Wars against Licinius

In the following years, Constantine gradually consolidated his military superiority over his rivals in the crumbling Tetrarchy. In 313, he met Licinius in Milan to secure their alliance by the marriage of Licinius and Constantine's half-sister Constantia. During this meeting, the emperors agreed on the so-called Edict of Milan, officially granting full tolerance to Christianity and all religions in the empire. The document had special benefits for Christians, legalizing their religion and granting them restoration for all property seized during Diocletian's persecution. It repudiates past methods of religious coercion and used only general terms to refer to the divine sphere—"Divinity" and "Supreme Divinity", summa divinitas. The conference was cut short, however, when news reached Licinius that his rival Maximinus had crossed the Bosporus and invaded European territory. Licinius departed and eventually defeated Maximinus, gaining control over the entire eastern half of the Roman Empire. Relations between the two remaining emperors deteriorated, as Constantine suffered an assassination attempt at the hands of a character that Licinius wanted elevated to the rank of Caesar; Licinius, for his part, had Constantine's statues in Emona destroyed.  In either 314 or 316, the two augusti fought against one another at the Battle of Cibalae, with Constantine being victorious. They clashed again at the Battle of Mardia in 317 and agreed to a settlement in which Constantine's sons Crispus and Constantine II, and Licinius' son Licinianus were made caesars.  After this arrangement, Constantine ruled the dioceses of Pannonia and Macedonia and took residence at Sirmium, whence he could wage war on the Goths and Sarmatians in 322, and on the Goths in 323, defeating and killing their leader Rausimod.
In 320, Licinius allegedly reneged on the religious freedom promised by the Edict of Milan and began to oppress Christians anew, generally without bloodshed, but resorting to confiscations and sacking of Christian office-holders. Although this characterization of Licinius as anti-Christian is somewhat doubtful, the fact is that he seems to have been far less open in his support of Christianity than Constantine. Therefore, Licinius was prone to see the Church as a force more loyal to Constantine than to the Imperial system in general, as the explanation offered by the Church historian Sozomen.
This dubious arrangement eventually became a challenge to Constantine in the West, climaxing in the great civil war of 324. Constantine's Christian eulogists present the war as a battle between Christianity and paganism; Licinius, aided by Gothic mercenaries, represented the past and ancient paganism, while Constantine and his Franks marched under the standard of the labarum. Outnumbered but fired by their zeal, Constantine's army emerged victorious in the Battle of Adrianople. Licinius fled across the Bosphorus and appointed Martinian, his magister officiorum, as nominal augustus in the West, but Constantine next won the Battle of the Hellespont and finally the Battle of Chrysopolis on 18 September 324. Licinius and Martinian surrendered to Constantine at Nicomedia on the promise their lives would be spared: they were sent to live as private citizens in Thessalonica and Cappadocia respectively, but in 325 Constantine accused Licinius of plotting against him and had them both arrested and hanged; Licinius' son (the son of Constantine's half-sister) was killed in 326. Thus Constantine became the sole emperor of the Roman Empire.

Later rule
Foundation of Constantinople



Diocletian had chosen Nicomedia in the East as his capital during the Tetrarchy—not far from Byzantium, well situated to defend Thrace, Asia, and Egypt, all of which had required his military attention. Constantine had recognised the shift of the empire from the remote and depopulated West to the richer cities of the East, and the military strategic importance of protecting the Danube from barbarian excursions and Asia from a hostile Persia in choosing his new capital as well as being able to monitor shipping traffic between the Black Sea and the Mediterranean. Licinius' defeat came to represent the defeat of a rival centre of pagan and Greek-speaking political activity in the East, as opposed to the Christian and Latin-speaking Rome, and it was proposed that a new Eastern capital should represent the integration of the East into the Roman Empire as a whole, as a centre of learning, prosperity, and cultural preservation for the whole of the Eastern Roman Empire. Among the various locations proposed for this alternative capital, Constantine appears to have toyed earlier with Serdica (present-day Sofia), as he was reported saying that "Serdica is my Rome". Sirmium and Thessalonica were also considered. Eventually, however, Constantine decided to work on the Greek city of Byzantium, which offered the advantage of having already been extensively rebuilt on Roman patterns of urbanism during the preceding century by Septimius Severus and Caracalla, who had already acknowledged its strategic importance. The city was thus founded in 324, dedicated on 11 May 330 and renamed Constantinopolis ("Constantine's City" or Constantinople in English). Special commemorative coins were issued in 330 to honor the event. The new city was protected by the relics of the True Cross, the Rod of Moses and other holy relics, though a cameo now at the Hermitage Museum also represented Constantine crowned by the tyche of the new city. The figures of old gods were either replaced or assimilated into a framework of Christian symbolism. Constantine built the new Church of the Holy Apostles on the site of a temple to Aphrodite. Generations later there was the story that a divine vision led Constantine to this spot, and an angel no one else could see led him on a circuit of the new walls. The capital would often be compared to the 'old' Rome as Nova Roma Constantinopolitana, the "New Rome of Constantinople".

Religious policy




Constantine was the first emperor to stop the persecution of Christians and to legalize Christianity, along with all other religions/cults in the Roman Empire. In February 313, he met with Licinius in Milan and developed the Edict of Milan, which stated that Christians should be allowed to follow their faith without oppression. This removed penalties for professing Christianity, under which many had been martyred previously, and it returned confiscated Church property. The edict protected all religions from persecution, not only Christianity, allowing anyone to worship any deity that they chose. A similar edict had been issued in 311 by Galerius, senior emperor of the Tetrarchy, which granted Christians the right to practise their religion but did not restore any property to them. The Edict of Milan included several clauses which stated that all confiscated churches would be returned, as well as other provisions for previously persecuted Christians. Some scholars think that Helena adopted Christianity as an adult, and according to Eusebius she was converted by Constantine, but other historians debate whether Constantine adopted his mother Helena's Christianity in his youth or whether he adopted it gradually over the course of his life.


Constantine possibly retained the title of pontifex maximus which emperors bore as heads of the ancient Roman religion until Gratian renounced the title. According to Christian writers, Constantine was over 40 when he finally declared himself a Christian, making it clear that he owed his successes to the protection of the Christian High God alone. Despite these declarations of being a Christian, he waited to be baptised on his deathbed, believing that the baptism would release him of any sins he committed in the course of carrying out his policies while emperor. He supported the Church financially, built basilicas, granted privileges to clergy (such as exemption from certain taxes), promoted Christians to high office, and returned property confiscated during the long period of persecution. His most famous building projects include the Church of the Holy Sepulchre and Old St. Peter's Basilica. In constructing the Old St. Peter's Basilica, Constantine went to great lengths to erect the basilica on top of St. Peter's resting place, so much so that it even affected the design of the basilica, including the challenge of erecting it on the hill where St. Peter rested, making its complete construction time over 30 years from the date Constantine ordered it to be built.
Constantine might not have patronised Christianity alone. A triumphal arch was built in 315 to celebrate his victory in the Battle of the Milvian Bridge which was decorated with images of the goddess Victoria, and sacrifices were made to pagan gods at its dedication, including Apollo, Diana, and Hercules. Absent from the arch are any depictions of Christian symbolism. However, the arch was commissioned by the Senate, so the absence of Christian symbols may reflect the role of the Curia at the time as a pagan redoubt.
In 321, he legislated that the venerable Sunday should be a day of rest for all citizens. In 323, he issued a decree banning Christians from participating in state sacrifices. After the pagan gods had disappeared from his coinage, Christian symbols appeared as Constantine's attributes, the chi rho between his hands or on his labarum, as well on the coinage. The reign of Constantine established a precedent for the emperor to have great influence and authority in the early Christian councils, most notably the dispute over Arianism. Constantine disliked the risks to societal stability that religious disputes and controversies brought with them, preferring to establish an orthodoxy. His influence over the Church councils was to enforce doctrine, root out heresy, and uphold ecclesiastical unity; the Church's role was to determine proper worship, doctrines, and dogma.
North African bishops struggled with Christian bishops who had been ordained by Donatus in opposition to Caecilian from 313 to 316. The African bishops could not come to terms, and the Donatists asked Constantine to act as a judge in the dispute. Three regional Church councils and another trial before Constantine all ruled against Donatus and the Donatism movement in North Africa. In 317, Constantine issued an edict to confiscate Donatist church property and to send Donatist clergy into exile. More significantly, in 325 he summoned the First Council of Nicaea, most known for its dealing with Arianism and for instituting the Nicene Creed. He enforced the council's prohibition against celebrating the Lord's Supper on the day before the Jewish Passover, which marked a definite break of Christianity from the Judaic tradition. From then on, the solar Julian calendar was given precedence over the lunisolar Hebrew calendar among the Christian churches of the Roman Empire.
Constantine made some new laws regarding the Jews; some of them were unfavourable towards Jews, although they were not harsher than those of his predecessors. It was made illegal for Jews to seek converts or to attack other Jews who had converted to Christianity. They were forbidden to own Christian slaves or to circumcise their slaves. On the other hand, Jewish clergy were given the same exemptions as Christian clergy.

Administrative reforms

Beginning in the mid-3rd century, the emperors began to favour members of the equestrian order over senators, who had a monopoly on the most important offices of the state. Senators were stripped of the command of legions and most provincial governorships, as it was felt that they lacked the specialised military upbringing needed in an age of acute defense needs; such posts were given to equestrians by Diocletian and his colleagues, following a practice enforced piecemeal by their predecessors. The emperors, however, still needed the talents and the help of the very rich, who were relied on to maintain social order and cohesion by means of a web of powerful influence and contacts at all levels. Exclusion of the old senatorial aristocracy threatened this arrangement.
In 326, Constantine reversed this pro-equestrian trend, raising many administrative positions to senatorial rank and thus opening these offices to the old aristocracy; at the same time, he elevated the rank of existing equestrian office-holders to senator, degrading the equestrian order in the process (at least as a bureaucratic rank). The title of perfectissimus was granted only to mid- or low-level officials by the end of the 4th century.
By the new Constantinian arrangement, one could become a senator by being elected praetor or by fulfilling a function of senatorial rank. From then on, holding actual power and social status were melded together into a joint imperial hierarchy. Constantine gained the support of the old nobility with this, as the Senate was allowed to elect praetors and quaestors in place of the usual practice of the emperors directly creating magistrates (adlectio). An inscription in honor of city prefect Ceionius Rufus Albinus states that Constantine had restored the Senate "the auctoritas it had lost at Caesar's time".
The Senate as a body remained devoid of any significant power; nevertheless, the senators had been marginalised as potential holders of imperial functions during the 3rd century but could dispute such positions alongside more upstart bureaucrats. Some modern historians see in those administrative reforms an attempt by Constantine at reintegrating the senatorial order into the imperial administrative elite to counter the possibility of alienating pagan senators from a Christianised imperial rule; however, such an interpretation remains conjectural, given the fact that we do not have the precise numbers about pre-Constantine conversions to Christianity in the old senatorial milieu. Some historians suggest that early conversions among the old aristocracy were more numerous than previously supposed.
Constantine's reforms had to do only with the civilian administration. The military chiefs had risen from the ranks since the Crisis of the Third Century but remained outside the Senate, in which they were included only by Constantine's children.

Monetary reforms

In the 3rd century, the production of fiat money to pay for public expenses resulted in runaway inflation, and Diocletian tried unsuccessfully to re-establish trustworthy minting of silver coins, as well as silver-bronze "billon" coins (the term "billon" meaning an alloy of precious and base metals that is mostly base metal). Silver currency was overvalued in terms of its actual metal content and therefore could only circulate at much discounted rates. Constantine stopped minting the Diocletianic "pure" silver argenteus soon after 305, while the "billon" currency continued to be used until the 360s. From the early 300s on, Constantine forsook any attempts at restoring the silver currency, preferring instead to concentrate on minting large quantities of the gold solidus, 72 of which made a pound of gold. New and highly debased silver pieces continued to be issued during his later reign and after his death, in a continuous process of retariffing, until this "billon" minting ceased in 367, and the silver piece was continued by various denominations of bronze coins, the most important being the centenionalis.
These bronze pieces continued to be devalued, assuring the possibility of keeping fiduciary minting alongside a gold standard. The author of De Rebus Bellicis held that the rift widened between classes because of this monetary policy; the rich benefited from the stability in purchasing power of the gold piece, while the poor had to cope with ever-degrading bronze pieces. Later emperors such as Julian the Apostate insisted on trustworthy mintings of the bronze currency.
Constantine's monetary policies were closely associated with his religious policies; increased minting was associated with the confiscation of all gold, silver, and bronze statues from pagan temples between 331 and 336 which were declared to be imperial property. Two imperial commissioners for each province had the task of getting the statues and melting them for immediate minting, with the exception of a number of bronze statues that were used as public monuments in Constantinople.

Executions of Crispus and Fausta

Constantine had his eldest son Crispus seized and put to death by "cold poison" at Pola (Pula, Croatia) sometime between 15 May and 17 June 326. In July, he had his wife Empress Fausta (stepmother of Crispus) killed in an overheated bath. Their names were wiped from the face of many inscriptions, references to their lives were eradicated from the literary record, and their memory was condemned. Eusebius, for example, edited out any praise of Crispus from later copies of Historia Ecclesiastica, and his Vita Constantini contains no mention of Fausta or Crispus. Few ancient sources are willing to discuss possible motives for the events, and the few that do are of later provenance and are generally unreliable. At the time of the executions, it was commonly believed that Empress Fausta was either in an illicit relationship with Crispus or was spreading rumors to that effect. A popular myth arose, modified to allude to the Hippolytus–Phaedra legend, with the suggestion that Constantine killed Crispus and Fausta for their immoralities; the largely fictional Passion of Artemius explicitly makes this connection. The myth rests on slim evidence as an interpretation of the executions; only late and unreliable sources allude to the relationship between Crispus and Fausta, and there is no evidence for the modern suggestion that Constantine's "godly" edicts of 326 and the irregularities of Crispus are somehow connected.
Although Constantine created his apparent heirs "caesars", following a pattern established by Diocletian, he gave his creations a hereditary character, alien to the tetrarchic system: Constantine's caesars were to be kept in the hope of ascending to empire and entirely subordinated to their augustus, as long as he was alive. Adrian Goldsworthy speculates an alternative explanation for the execution of Crispus was Constantine's desire to keep a firm grip on his prospective heirs, this—and Fausta's desire for having her sons inheriting instead of their half-brother—being reason enough for killing Crispus; the subsequent execution of Fausta, however, was probably meant as a reminder to her children that Constantine would not hesitate in "killing his own relatives when he felt this was necessary".

Later campaigns

Constantine considered Constantinople his capital and permanent residence. He lived there for a good portion of his later life. In 328, construction was completed on Constantine's Bridge at Sucidava, (today Celei in Romania) in hopes of reconquering Dacia, a province that had been abandoned under Aurelian. In the late winter of 332, Constantine campaigned with the Sarmatians against the Goths. The weather and lack of food reportedly cost the Goths dearly before they submitted to Rome. In 334, after Sarmatian commoners had overthrown their leaders, Constantine led a campaign against the tribe. He won a victory in the war and extended his control over the region, as remains of camps and fortifications in the region indicate. Constantine resettled some Sarmatian exiles as farmers in Illyrian and Roman districts and conscripted the rest into the army. Constantine reconquered the South of Dacia and the new frontier in Dacia was along the wall and ditch called Brazda lui Novac line supported by new castra. Constantine took the title Dacicus maximus in 336.
In the last years of his life, Constantine made plans for a campaign against Persia. In a letter written to the king of Persia, Shapur, Constantine had asserted his patronage over Persia's Christian subjects and urged Shapur to treat them well. The letter is undatable. In response to border raids, Constantine sent Constantius to guard the eastern frontier in 335. In 336, Prince Narseh invaded Armenia (a Christian kingdom since 301) and installed a Persian client on the throne. Constantine then resolved to campaign against Persia. He treated the war as a Christian crusade, calling for bishops to accompany the army and commissioning a tent in the shape of a church to follow him everywhere. Constantine planned to be baptised in the Jordan River before crossing into Persia. Persian diplomats came to Constantinople over the winter of 336–337, seeking peace, but Constantine turned them away. The campaign was called off, however, when Constantine became sick in the spring of 337.

Illness and death


From his recent illness, Constantine knew death would soon come. Within the Church of the Holy Apostles, Constantine had secretly prepared a final resting-place for himself. It came sooner than he had expected. Soon after the Feast of Easter 337, Constantine fell seriously ill. He left Constantinople for the hot baths near his mother's city of Helenopolis (Altınova), on the southern shores of the Gulf of Nicomedia (present-day Gulf of İzmit). There, in a church his mother built in honor of Lucian the Martyr, he prayed, and there he realised that he was dying. Seeking purification, he became a catechumen and attempted a return to Constantinople, making it only as far as a suburb of Nicomedia. He summoned the bishops and told them of his hope to be baptised in the River Jordan, where Christ was written to have been baptised. He requested the baptism right away, promising to live a more Christian life should he live through his illness. The bishops, Eusebius records, "performed the sacred ceremonies according to custom". He chose the Arian bishop Eusebius of Nicomedia, bishop of the city where he lay dying, as his baptizer. In postponing his baptism, he followed one custom at the time which postponed baptism until after infancy. It has been thought that Constantine put off baptism as long as he did so as to be absolved from as much of his sin as possible. Constantine died soon after at a suburban villa called Achyron, on the last day of the fifty-day festival of Pentecost directly following Pascha (or Easter), on 22 May 337.
Although Constantine's death follows the conclusion of the Persian campaign in Eusebius's account, most other sources report his death as occurring in its middle. Emperor Julian (a nephew of Constantine), writing in the mid-350s, observes that the Sassanians escaped punishment for their ill-deeds, because Constantine died "in the middle of his preparations for war". Similar accounts are given in the Origo Constantini, an anonymous document composed while Constantine was still living, which has Constantine dying in Nicomedia; the Historiae abbreviatae of Sextus Aurelius Victor, written in 361, which has Constantine dying at an estate near Nicomedia called Achyrona while marching against the Persians; and the Breviarium of Eutropius, a handbook compiled in 369 for the Emperor Valens, which has Constantine dying in a nameless state villa in Nicomedia. From these and other accounts, some have concluded that Eusebius's Vita was edited to defend Constantine's reputation against what Eusebius saw as a less congenial version of the campaign.
Following his death, his body was transferred to Constantinople and buried in the Church of the Holy Apostles, in a porphyry sarcophagus that was described in the 10th century by Constantine VII Porphyrogenitus in the De Ceremoniis. His body survived the plundering of the city during the Fourth Crusade in 1204 but was destroyed at some point afterwards. Constantine was succeeded by his three sons born of Fausta, Constantine II, Constantius II and Constans. His sons, along with his nephew Dalmatius, had already received one division of the empire each to administer as caesars; Constantine may have intended his successors to resume a structure akin to Diocletian's Tetrarchy. A number of relatives were killed by followers of Constantius, notably Constantine's nephews Dalmatius (who held the rank of caesar) and Hannibalianus, presumably to eliminate possible contenders to an already complicated succession. He also had two daughters, Constantina and Helena, wife of Emperor Julian.

Assessment and legacy

Constantine reunited the empire under one emperor, and he won major victories over the Franks and Alamanni in 306–308, the Franks again in 313–314, the Goths in 332, and the Sarmatians in 334. By 336, he had reoccupied most of the long-lost province of Dacia which Aurelian had been forced to abandon in 271. At the time of his death, he was planning a great expedition to end raids on the eastern provinces from the Persian Empire.
In the cultural sphere, Constantine revived the clean-shaven face fashion of earlier emperors, originally introduced among the Romans by Scipio Africanus (236–183 BC) and changed into the wearing of the beard by Hadrian (r. 117–138). With the exception of Julian the Apostate (r. 360–363), this new Roman imperial fashion lasted until the reign of Phocas (r. 602–610) in the 7th century.
The Holy Roman Empire reckoned Constantine among the venerable figures of its tradition. In the later Byzantine state, it became a great honor for an emperor to be hailed as a "new Constantine"; ten emperors carried the name, including the last emperor of the Eastern Roman Empire. Charlemagne used monumental Constantinian forms in his court to suggest that he was Constantine's successor and equal. Charlemagne, Henry VIII, Philip II of Spain, Godfrey of Bouillon, House of Capet, House of Habsburg, House of Stuart, Macedonian dynasty and Phokas family claimed descent from Constantine. Geoffrey of Monmouth embroidered a tale that the legendary king of Britain, King Arthur, was also a descendant of Constantine. Constantine acquired a mythic role as a hero and warrior against heathens. His reception as a saint seems to have spread within the Byzantine empire during wars against the Sasanian Persians and the Muslims in the late 6th and 7th century. The motif of the Romanesque equestrian, the mounted figure in the posture of a triumphant Roman emperor, became a visual metaphor in statuary in praise of local benefactors. The name "Constantine" enjoyed renewed popularity in western France in the 11th and 12th centuries. During the Fascist period in Italy in the 20th century, parallels between Constantine and Mussolini became especially popular after the signing of the Lateran Pacts by the Italian State and the Catholic Church in 1929. Mussolini's perceived role in bringing about the historic agreement was sometimes even explicitly compared to Constantine's Edict of Milan. For example, the archbishop of Milan, Cardinal Ildefonso Schuster, claimed that, after sixteen centuries, a second March on Rome had occurred and a second 'religious pact' had been established, linking Mussolini to the spiriti magni of both Constantine and Augustus.
The Niš Constantine the Great Airport is named in honor of him. A large cross was planned to be built on a hill overlooking Niš, but the project was cancelled. In 2012, a memorial was erected in Niš in his honor. The Commemoration of the Edict of Milan was held in Niš in 2013. The Orthodox Church considers Constantine a saint (Άγιος Κωνσταντίνος, Saint Constantine), having a feast day on 21 May, and calls him isapostolos (ισαπόστολος Κωνσταντίνος)—an equal of the Apostles.

Historiography

During Constantine's lifetime, Praxagoras of Athens and Libanius, pagan authors, showered Constantine with praise, presenting him as a paragon of virtue. His nephew and son-in-law Julian the Apostate, however, wrote the satire Symposium, or the Saturnalia in 361, after the last of his sons died; it denigrated Constantine, calling him inferior to the great pagan emperors, and given over to luxury and greed. Following Julian, Eunapius began – and Zosimus continued – a historiographic tradition that blamed Constantine for weakening the empire through his indulgence to the Christians.
During the Middle Ages, European and Near-East Byzantine writers presented Constantine as an ideal ruler, the standard against which any king or emperor could be measured. The Renaissance rediscovery of anti-Constantinian sources prompted a re-evaluation of his career. German humanist Johannes Leunclavius discovered Zosimus' writings and published a Latin translation in 1576. In its preface, he argues that Zosimus' picture of Constantine offered a more balanced view than that of Eusebius and the Church historians. Cardinal Caesar Baronius criticised Zosimus, favouring Eusebius' account of the Constantinian era. Baronius' Life of Constantine (1588) presents Constantine as the model of a Christian prince. Edward Gibbon aimed to unite the two extremes of Constantinian scholarship in his work The History of the Decline and Fall of the Roman Empire (1776–1789) by contrasting the portraits presented by Eusebius and Zosimus. He presents a noble war hero who transforms into an Oriental despot in his old age, "degenerating into a cruel and dissolute monarch".
Modern interpretations of Constantine's rule begin with Jacob Burckhardt's The Age of Constantine the Great (1853, rev. 1880). Burckhardt's Constantine is a scheming secularist, a politician who manipulates all parties in a quest to secure his own power. Henri Grégoire followed Burckhardt's evaluation of Constantine in the 1930s, suggesting that Constantine developed an interest in Christianity only after witnessing its political usefulness. Grégoire was skeptical of the authenticity of Eusebius's Vita, and postulated a pseudo-Eusebius to assume responsibility for the vision and conversion narratives of that work. Otto Seeck's Geschichte des Untergangs der antiken Welt (1920–1923) and André Piganiol's L'empereur Constantin (1932) go against this historiographic tradition. Seeck presents Constantine as a sincere war hero whose ambiguities were the product of his own naïve inconsistency. Piganiol's Constantine is a philosophical monotheist, a child of his era's religious syncretism. Related histories by Arnold Hugh Martin Jones (Constantine and the Conversion of Europe, 1949) and Ramsay MacMullen (Constantine, 1969) give portraits of a less visionary and more impulsive Constantine.
These later accounts were more willing to present Constantine as a genuine convert to Christianity. Norman H. Baynes began a historiographic tradition with Constantine the Great and the Christian Church (1929) which presents Constantine as a committed Christian, reinforced by Andreas Alföldi's The Conversion of Constantine and Pagan Rome (1948), and Timothy Barnes's Constantine and Eusebius (1981) is the culmination of this trend. Barnes' Constantine experienced a radical conversion which drove him on a personal crusade to convert his empire. Charles Matson Odahl's Constantine and the Christian Empire (2004) takes much the same tack. In spite of Barnes' work, arguments continue over the strength and depth of Constantine's religious conversion. Certain themes in this school reached new extremes in T. G. Elliott's The Christianity of Constantine the Great (1996), which presented Constantine as a committed Christian from early childhood. Paul Veyne's 2007 work Quand notre monde est devenu chrétien holds a similar view which does not speculate on the origin of Constantine's Christian motivation, but presents him as a religious revolutionary who fervently believed that he was meant "to play a providential role in the millenary economy of the salvation of humanity". Peter Heather argues that it is most plausible that Constantine had been a Christian considerably before 312 – possibly even for his entire life – with the public timeline of events instead reflecting his "coming out" as Christian in stages as doing so became politically viable. As a parallel illustrating the cogency of this interpretation, Heather gestures to the later conversion of Constantine's nephew Julian from Christianity to Hellenism, after which he practiced in secret for a decade.

Donation of Constantine

Latin Christians considered it inappropriate that Constantine was baptised only on his death bed by an unorthodox bishop, and a legend emerged by the early 4th century that Pope Sylvester I had cured the pagan emperor from leprosy. According to this legend, Constantine was baptised and began the construction of a church in the Lateran Basilica. The Donation of Constantine appeared in the 8th century, most likely during the pontificate of Pope Stephen II, in which the freshly converted Constantine gives "the city of Rome and all the provinces, districts, and cities of Italy and the Western regions" to Sylvester and his successors. In the High Middle Ages, this document was used and accepted as the basis for the pope's temporal power, though it was denounced as a forgery by Emperor Otto III and lamented as the root of papal worldliness by Dante Alighieri. Philologist and Catholic priest Lorenzo Valla proved in 1440 that the document was indeed a forgery.

Geoffrey of Monmouth's Historia
During the medieval period, Britons regarded Constantine as a king of their own people, particularly associating him with Caernarfon in Gwynedd. While some of this is owed to his fame and his proclamation as emperor in Britain, there was also confusion of his family with Magnus Maximus's supposed wife Elen and her son, another Constantine (Welsh: Custennin). In the 12th century Henry of Huntingdon included a passage in his Historia Anglorum that the Emperor Constantine's mother was a Briton, making her the daughter of King Cole of Colchester. Geoffrey of Monmouth expanded this story in his highly fictionalised Historia Regum Britanniae, an account of the supposed Kings of Britain from their Trojan origins to the Anglo-Saxon invasion. According to Geoffrey, Cole was King of the Britons when Constantius, here a senator, came to Britain. Afraid of the Romans, Cole submits to Roman law so long as he retains his kingship. However, he dies only a month later, and Constantius takes the throne himself, marrying Cole's daughter Helena. They have their son Constantine, who succeeds his father as King of Britain before becoming Roman emperor.
Historically, this series of events is extremely improbable. Constantius had already left Helena by the time he left for Britain. Additionally, no earlier source mentions that Helena was born in Britain, let alone that she was a princess. Henry's source for the story is unknown, though it may have been a lost hagiography of Helena.

Family tree





See also

Bronze colossus of Constantine
Colossus of Constantine
Fifty Bibles of Constantine
German and Sarmatian campaigns of Constantine
Life of Constantine
List of Byzantine emperors
List of people known as the great
Notes

References
Citations

Sources
Ancient sources

Modern sources

Further reading
Arjava, Antii. Women and Law in Late Antiquity. Oxford: Oxford University Press, 1996. ISBN 0-19-815233-7
Baynes, Norman H. (1930). Constantine the Great and the Christian Church. London: Milford.
Burckhardt, Jacob (1949). The Age of Constantine the Great. London: Routledge.
Cameron, Averil (1993). The later Roman empire: AD 284–430. London: Fontana Press. ISBN 978-0-00-686172-0.
Cowan, Ross (2016). Milvian Bridge AD 312: Constantine's Battle for Empire and Faith. Oxford: Osprey Publishing.
Eadie, John W., ed. (1971). The conversion of Constantine. New York: Holt, Rinehart and Winston. ISBN 978-0-03-083645-9.
Fourlas, Benjamin (2020). "St Constantine and the Army of Heroic Men Raised by Tiberius II Constantine in 574/575. Some Thoughts on the Historical Significance of the Early Byzantine Silver Hoard at Karlsruhe". Jahrbuch des Römisch-Germanischen Zentralmuseums 62, 2015 [published 2020], 341–375. doi:10.11588/jrgzm.2015.1.77142
Harries, Jill. Law and Empire in Late Antiquity. Cambridge, UK: Cambridge University Press, 2004. Hardcover ISBN 0-521-41087-8 Paperback ISBN 0-521-42273-6
Hartley, Elizabeth. Constantine the Great: York's Roman Emperor. York: Lund Humphries, 2004. ISBN 978-0-85331-928-3.
Heather, Peter J. "Foedera and Foederati of the Fourth Century." In From Roman Provinces to Medieval Kingdoms, edited by Thomas F.X. Noble, 292–308. New York: Routledge, 2006. Hardcover ISBN 0-415-32741-5 Paperback ISBN 0-415-32742-3
Leithart, Peter J. Defending Constantine: The Twilight of an Empire and the Dawn of Christendom. Downers Grove: IL, InterVarsity Press 2010
MacMullen, Ramsay. Christianizing the Roman Empire A.D. 100–400. New Haven, CT; London: Yale University Press, 1984. ISBN 978-0-300-03642-8
MacMullen, Ramsay. Christianity and Paganism in the Fourth to Eighth Centuries. New Haven: Yale University Press, 1997. ISBN 0-300-07148-5
Percival J. On the Question of Constantine's Conversion to Christianity Archived 14 June 2015 at the Wayback Machine, Clio History Journal, 2008
Pelikán, Jaroslav (1987). The excellent empire: the fall of Rome and the triumph of the church. San Francisco: Harper &amp; Row. ISBN 978-0-06-254636-4.
Velikov, Yuliyan (2013). Imperator et Sacerdos. Veliko Turnovo University Press. ISBN 978-954-524-932-7 (in Bulgarian)
External links



Complete chronological list of Constantine's extant writings (archived 19 February 2013)
Firth, John B. "Constantine the Great, the Reorganisation of the Empire and the Triumph of the Church". Archived from the original (BTM) on 15 March 2012. Retrieved 19 February 2016.
Letters of Constantine: Book 1, Book 2, &amp; Book 3
Encyclopædia Britannica, Constantine I
Henry Stuart Jones (1911). "Constantine (emperors)". In Chisholm, Hugh (ed.). Encyclopædia Britannica. 6. (11th ed.), Cambridge University Press. pp. 988–992.
Charles George Herbermann and Georg Grupp (1908). "Constantine the Great". In Catholic Encyclopedia. 4. New York: Robert Appleton Company.
BBC North Yorkshire's site on Constantine the Great
Constantine's time in York on the 'History of York'
Commemorations
Roman Legionary AD 284–337: The Age of Diocletian and Constantine the Great
Milvian Bridge AD 312: Constantine's Battle for Empire and Faith
"##;
