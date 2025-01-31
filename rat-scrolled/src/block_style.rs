//!
//! Defines [BlockStyle] which can be used to define styles for a block.
//!
//! ```rust
//! use ratatui::style::{Style, Stylize};
//! use ratatui::widgets::{Block, Borders};
//! use rat_scrolled::block_style::{BlockStyle, StyleBorderType, StylizeBlock};
//!
//! let s = BlockStyle {
//!     borders: Some(Borders::LEFT|Borders::RIGHT),
//!     border_style: Some(Style::new().yellow().on_black()),
//!     border_type: Some(StyleBorderType::Plain),
//!     ..Default::default()
//! };
//!
//! // ... later ...
//!
//! Block::new()
//!     .styles(s);
//! //  .render(...);
//!
//! ```
//!
//! This is useful if you have a central definition of all the styles
//! in the application.
//!
//! BlockStyle can be serialized too.
//!
use crate::_private::{NonExhaustive, Sealed};
use ratatui::layout::Alignment;
use ratatui::style::Style;
use ratatui::symbols::border::{
    EMPTY, FULL, ONE_EIGHTH_TALL, ONE_EIGHTH_WIDE, PROPORTIONAL_TALL, PROPORTIONAL_WIDE,
};
use ratatui::widgets::block::Position;
use ratatui::widgets::{Block, BorderType, Borders, Padding};
#[cfg(feature = "serde")]
use serde_derive::{Deserialize, Serialize};

/// __UNSTABLE__
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum StyleBorderType {
    Plain,
    Rounded,
    Double,
    Thick,
    QuadrantOutside,
    QuadrantInside,
    OneEighthWide,
    OneEighthTall,
    ProportionalWide,
    ProportionalTall,
    Full,
    Empty,
}

/// __UNSTABLE__
#[derive(Debug, Clone)]
pub struct BlockStyle {
    pub titles_style: Option<Style>,
    pub titles_alignment: Option<Alignment>,
    pub titles_position: Option<Position>,
    pub borders: Option<Borders>,
    pub border_style: Option<Style>,
    pub border_type: Option<StyleBorderType>,
    pub padding: Option<Padding>,

    pub non_exhaustive: NonExhaustive,
}

/// Extension trait for Block and Option<Block>.
pub trait StylizeBlock: Sealed {
    /// Set all block styles.
    fn styles(self, styles: BlockStyle) -> Self;
}

impl Sealed for Block<'_> {}

impl Sealed for Option<Block<'_>> {}

impl StylizeBlock for Option<Block<'_>> {
    fn styles(self, styles: BlockStyle) -> Self {
        if let Some(block) = self {
            Some(block.styles(styles))
        } else {
            None
        }
    }
}

impl StylizeBlock for Block<'_> {
    fn styles(mut self, styles: BlockStyle) -> Self {
        if let Some(s) = styles.titles_style {
            self = self.title_style(s);
        }
        if let Some(s) = styles.titles_alignment {
            self = self.title_alignment(s);
        }
        if let Some(s) = styles.titles_position {
            self = self.title_position(s);
        }
        if let Some(s) = styles.borders {
            self = self.borders(s);
        }
        if let Some(s) = styles.border_style {
            self = self.border_style(s);
        }
        if let Some(s) = styles.border_type {
            self = match s {
                StyleBorderType::Plain => self.border_type(BorderType::Plain),
                StyleBorderType::Rounded => self.border_type(BorderType::Rounded),
                StyleBorderType::Double => self.border_type(BorderType::Double),
                StyleBorderType::Thick => self.border_type(BorderType::Thick),
                StyleBorderType::QuadrantOutside => self.border_type(BorderType::QuadrantOutside),
                StyleBorderType::QuadrantInside => self.border_type(BorderType::QuadrantInside),
                StyleBorderType::OneEighthWide => self.border_set(ONE_EIGHTH_WIDE),
                StyleBorderType::OneEighthTall => self.border_set(ONE_EIGHTH_TALL),
                StyleBorderType::ProportionalWide => self.border_set(PROPORTIONAL_WIDE),
                StyleBorderType::ProportionalTall => self.border_set(PROPORTIONAL_TALL),
                StyleBorderType::Full => self.border_set(FULL),
                StyleBorderType::Empty => self.border_set(EMPTY),
            }
        }

        self
    }
}

impl Default for BlockStyle {
    fn default() -> Self {
        Self {
            titles_style: Default::default(),
            titles_alignment: Default::default(),
            titles_position: Default::default(),
            borders: Default::default(),
            border_style: Default::default(),
            border_type: Default::default(),
            padding: Default::default(),
            non_exhaustive: NonExhaustive,
        }
    }
}

#[cfg(feature = "serde")]
mod padding {
    use ratatui::widgets::Padding;
    use serde_derive::{Deserialize, Serialize};

    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    #[cfg_attr(feature = "serde", serde(rename = "Padding"))]
    pub struct PaddingSerde {
        pub left: u16,
        pub right: u16,
        pub top: u16,
        pub bottom: u16,
    }

    impl From<Padding> for PaddingSerde {
        fn from(value: Padding) -> Self {
            Self {
                left: value.left,
                right: value.right,
                top: value.top,
                bottom: value.bottom,
            }
        }
    }

    impl From<PaddingSerde> for Padding {
        fn from(value: PaddingSerde) -> Self {
            Padding {
                left: value.left,
                right: value.right,
                top: value.top,
                bottom: value.bottom,
            }
        }
    }
}

#[cfg(feature = "serde")]
pub mod ser {
    use crate::block_style::padding::PaddingSerde;
    use crate::block_style::BlockStyle;
    use ratatui::widgets::Borders;

    impl serde::Serialize for BlockStyle {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            use serde::ser::SerializeStruct;

            let mut blockstyle = serializer.serialize_struct("BlockStyle", 7)?;
            blockstyle.serialize_field("titles_style", &self.titles_style)?;
            blockstyle.serialize_field(
                "titles_alignment",
                &self.titles_alignment.map(|v| v.to_string()),
            )?;
            blockstyle.serialize_field(
                "titles_position",
                &self.titles_position.map(|v| v.to_string()),
            )?;
            blockstyle.serialize_field("borders", &self.borders.map(|v| borders_str(v)))?;
            blockstyle.serialize_field("border_style", &self.border_style)?;
            blockstyle.serialize_field("border_type", &self.border_type)?;
            blockstyle.serialize_field("padding", &self.padding.map(|v| PaddingSerde::from(v)))?;
            blockstyle.end()
        }
    }

    pub fn borders_str(borders: Borders) -> String {
        let mut tmp = String::new();
        if borders.contains(Borders::TOP) {
            tmp.push('t');
        }
        if borders.contains(Borders::RIGHT) {
            tmp.push('r');
        }
        if borders.contains(Borders::BOTTOM) {
            tmp.push('b');
        }
        if borders.contains(Borders::LEFT) {
            tmp.push('l');
        }
        tmp
    }

    pub mod alignment {
        use ratatui::layout::Alignment;

        #[cfg(feature = "serde")]
        pub fn serialize<S: serde::Serializer>(
            value: &Alignment,
            serializer: S,
        ) -> Result<S::Ok, S::Error> {
            use serde::Serialize;

            let str = value.to_string();
            str.serialize(serializer)
        }
    }
}

#[cfg(feature = "serde")]
pub mod de {
    use crate::block_style::padding::PaddingSerde;
    use crate::block_style::BlockStyle;
    use ratatui::layout::Alignment;
    use ratatui::widgets::block::Position;
    use ratatui::widgets::Borders;
    use std::fmt;
    use std::str::FromStr;

    impl<'de> serde::Deserialize<'de> for BlockStyle {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_struct("BlockStyle", FIELDS, BlockStyleVisitor)
        }
    }

    pub const FIELDS: &[&str] = &[
        "titles_style",
        "titles_alignment",
        "titles_position",
        "borders",
        "border_style",
        "border_type",
        "padding",
    ];

    enum Field {
        TitlesStyle,
        TitlesAlignment,
        TitlesPosition,
        Borders,
        BorderStyle,
        BorderType,
        Padding,
    }

    impl<'de> serde::Deserialize<'de> for Field {
        fn deserialize<D>(deserializer: D) -> Result<Field, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            struct FieldVisitor;

            impl<'de> serde::de::Visitor<'de> for FieldVisitor {
                type Value = Field;

                fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                    formatter.write_str("blingdaks!")
                }

                fn visit_str<E>(self, value: &str) -> Result<Field, E>
                where
                    E: serde::de::Error,
                {
                    match value {
                        "titles_style" => Ok(Field::TitlesStyle),
                        "titles_alignment" => Ok(Field::TitlesAlignment),
                        "titles_position" => Ok(Field::TitlesPosition),
                        "borders" => Ok(Field::Borders),
                        "border_style" => Ok(Field::BorderStyle),
                        "border_type" => Ok(Field::BorderType),
                        "padding" => Ok(Field::Padding),
                        _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                    }
                }
            }

            deserializer.deserialize_identifier(FieldVisitor)
        }
    }

    pub struct BlockStyleVisitor;

    impl<'de> serde::de::Visitor<'de> for BlockStyleVisitor {
        type Value = BlockStyle;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("struct BlockStyle")
        }

        fn visit_seq<V>(self, mut seq: V) -> Result<BlockStyle, V::Error>
        where
            V: serde::de::SeqAccess<'de>,
        {
            let titles_style = seq
                .next_element()?
                .ok_or_else(|| serde::de::Error::invalid_length(0, &self))?;
            let titles_alignment = seq
                .next_element::<Option<String>>()?
                .ok_or_else(|| serde::de::Error::invalid_length(1, &self))?;
            let titles_alignment = if let Some(v) = titles_alignment {
                match Alignment::from_str(&v) {
                    Ok(v) => Some(v),
                    Err(e) => Err(serde::de::Error::custom(format!("{:?}", e)))?,
                }
            } else {
                None
            };
            let titles_position = seq
                .next_element::<Option<String>>()?
                .ok_or_else(|| serde::de::Error::invalid_length(2, &self))?;
            let titles_position = if let Some(v) = titles_position {
                match Position::from_str(&v) {
                    Ok(v) => Some(v),
                    Err(e) => Err(serde::de::Error::custom(format!("{:?}", e)))?,
                }
            } else {
                None
            };
            let borders = seq
                .next_element::<Option<String>>()?
                .ok_or_else(|| serde::de::Error::invalid_length(3, &self))?;
            let borders = if let Some(v) = borders {
                Some(str_borders(&v))
            } else {
                None
            };
            let border_style = seq
                .next_element()?
                .ok_or_else(|| serde::de::Error::invalid_length(4, &self))?;
            let border_type = seq
                .next_element()?
                .ok_or_else(|| serde::de::Error::invalid_length(5, &self))?;
            let padding = seq
                .next_element::<Option<PaddingSerde>>()?
                .ok_or_else(|| serde::de::Error::invalid_length(6, &self))?;
            let padding = padding.map(|v| v.into());

            Ok(BlockStyle {
                titles_style,
                titles_alignment,
                titles_position,
                borders,
                border_style,
                border_type,
                padding,
                ..Default::default()
            })
        }

        fn visit_map<V>(self, mut map: V) -> Result<BlockStyle, V::Error>
        where
            V: serde::de::MapAccess<'de>,
        {
            let mut titles_style = None;
            let mut titles_alignment = None;
            let mut titles_position = None;
            let mut borders = None;
            let mut border_style = None;
            let mut border_type = None;
            let mut padding = None;
            while let Some(key) = map.next_key()? {
                match key {
                    Field::TitlesStyle => {
                        if titles_style.is_some() {
                            return Err(serde::de::Error::duplicate_field("titles_style"));
                        }
                        titles_style = Some(map.next_value()?);
                    }
                    Field::TitlesAlignment => {
                        if titles_alignment.is_some() {
                            return Err(serde::de::Error::duplicate_field("titles_alignment"));
                        }
                        let tmp = map.next_value::<Option<String>>()?;
                        titles_alignment = if let Some(v) = tmp {
                            match Alignment::from_str(&v) {
                                Ok(v) => Some(Some(v)),
                                Err(e) => Err(serde::de::Error::custom(format!("{:?}", e)))?,
                            }
                        } else {
                            None
                        };
                    }
                    Field::TitlesPosition => {
                        if titles_position.is_some() {
                            return Err(serde::de::Error::duplicate_field("titles_position"));
                        }
                        let tmp = map.next_value::<Option<String>>()?;
                        titles_position = if let Some(v) = tmp {
                            match Position::from_str(&v) {
                                Ok(v) => Some(Some(v)),
                                Err(e) => Err(serde::de::Error::custom(format!("{:?}", e)))?,
                            }
                        } else {
                            None
                        };
                    }
                    Field::Borders => {
                        if borders.is_some() {
                            return Err(serde::de::Error::duplicate_field("borders"));
                        }
                        let tmp = map.next_value::<Option<String>>()?;
                        borders = Some(if let Some(v) = tmp {
                            Some(str_borders(&v))
                        } else {
                            None
                        })
                    }
                    Field::BorderStyle => {
                        if border_style.is_some() {
                            return Err(serde::de::Error::duplicate_field("border_style"));
                        }
                        border_style = Some(map.next_value()?);
                    }
                    Field::BorderType => {
                        if border_type.is_some() {
                            return Err(serde::de::Error::duplicate_field("border_type"));
                        }
                        border_type = Some(map.next_value()?);
                    }
                    Field::Padding => {
                        if padding.is_some() {
                            return Err(serde::de::Error::duplicate_field("padding"));
                        }
                        let tmp = map.next_value::<Option<PaddingSerde>>()?;
                        padding = Some(tmp.map(|v| v.into()));
                    }
                }
            }
            Ok(BlockStyle {
                titles_style: titles_style.unwrap_or_default(),
                titles_alignment: titles_alignment.unwrap_or_default(),
                titles_position: titles_position.unwrap_or_default(),
                borders: borders.unwrap_or_default(),
                border_style: border_style.unwrap_or_default(),
                border_type: border_type.unwrap_or_default(),
                padding: padding.unwrap_or_default(),
                ..Default::default()
            })
        }
    }

    pub fn str_borders(str: &str) -> Borders {
        let mut borders = Borders::empty();
        for c in str.chars() {
            if c == 't' {
                borders.set(Borders::TOP, true);
            }
            if c == 'r' {
                borders.set(Borders::TOP, true);
            }
            if c == 'b' {
                borders.set(Borders::TOP, true);
            }
            if c == 'l' {
                borders.set(Borders::TOP, true);
            }
        }
        borders
    }
}

#[cfg(all(feature = "serde", test))]
mod tests {
    use crate::block_style::{BlockStyle, StyleBorderType};
    use ratatui::layout::Alignment;
    use ratatui::style::{Style, Stylize};
    use ratatui::widgets::block::Position;
    use ratatui::widgets::Padding;

    #[test]
    fn test_serde() {
        let style = BlockStyle {
            titles_style: Some(Style::new().black().underlined()),
            titles_alignment: Some(Alignment::Center),
            titles_position: Some(Position::Top),
            borders: Default::default(),
            border_style: Some(Style::new().not_bold()),
            border_type: Some(StyleBorderType::Plain),
            padding: Some(Padding::new(1, 2, 3, 4)),
            ..Default::default()
        };

        let str = serde_json::to_string_pretty(&style).unwrap();
        println!("{}", str);

        let v: BlockStyle = serde_json::from_str(&str).unwrap();
        println!("{:?}", v);
    }
}
