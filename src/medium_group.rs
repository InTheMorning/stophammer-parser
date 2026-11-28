//! The map from a `podcast:medium` value to its medium group.
//!
//! ADR 0110 §1.1 defines the groups and their mediums. This module is the
//! only place that compares a medium text with a group (ADR 0110 §1.5).
//! ADR 0110 §8.2 defines the reader type of the `mediums` field of
//! `/node/info`.

/// A medium group (ADR 0110 §1.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MediumGroup {
    /// The mediums `music` and `musicL`.
    Music,
    /// The mediums `podcast` and `podcastL`.
    Podcast,
}

impl MediumGroup {
    /// Each group, in a fixed order.
    pub const ALL: [MediumGroup; 2] = [MediumGroup::Music, MediumGroup::Podcast];

    /// The lower-case name of the group.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            MediumGroup::Music => "music",
            MediumGroup::Podcast => "podcast",
        }
    }

    /// The group with this exact lower-case name.
    #[must_use]
    pub fn from_name(name: &str) -> Option<MediumGroup> {
        match name {
            "music" => Some(MediumGroup::Music),
            "podcast" => Some(MediumGroup::Podcast),
            _ => None,
        }
    }

    /// The mediums of the group. The first medium is the default medium.
    #[must_use]
    pub fn mediums(self) -> &'static [&'static str] {
        match self {
            MediumGroup::Music => &["music", "musicL"],
            MediumGroup::Podcast => &["podcast", "podcastL"],
        }
    }

    /// The default medium of the group: the first medium.
    #[must_use]
    pub fn default_medium(self) -> &'static str {
        self.mediums()[0]
    }
}

/// The class of a raw `podcast:medium` value (ADR 0110 §1.1 to §1.4).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MediumClass {
    /// The medium is in this group.
    Group(MediumGroup),
    /// The medium is `publisher`. It is in no group (ADR 0110 §1.2).
    Publisher,
    /// The medium is in no group, for example `video` (ADR 0110 §1.3).
    NoGroup,
    /// The feed has no medium, or the medium text is empty (ADR 0110 §1.4).
    Absent,
}

/// Classify a raw medium value.
///
/// The comparison ignores case and the white space at the two ends.
#[must_use]
pub fn classify(raw_medium: Option<&str>) -> MediumClass {
    let Some(raw) = raw_medium else {
        return MediumClass::Absent;
    };
    let text = raw.trim();
    if text.is_empty() {
        return MediumClass::Absent;
    }
    if text.eq_ignore_ascii_case("publisher") {
        return MediumClass::Publisher;
    }
    for group in MediumGroup::ALL {
        if group
            .mediums()
            .iter()
            .any(|medium| medium.eq_ignore_ascii_case(text))
        {
            return MediumClass::Group(group);
        }
    }
    MediumClass::NoGroup
}

/// The admission value of a group (ADR 0110 §3, §8.1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Admission {
    /// The node admits each feed of the group.
    Open,
    /// The node admits only the feeds of its list.
    List,
    /// An admission value that this release does not know.
    ///
    /// A later record can add a value. A reader must not fail on it.
    Other(String),
}

impl Admission {
    /// Read an admission value. An unknown value becomes `Other`.
    #[must_use]
    pub fn parse(text: &str) -> Admission {
        match text {
            "open" => Admission::Open,
            "list" => Admission::List,
            other => Admission::Other(other.to_owned()),
        }
    }

    /// The text of the admission value.
    #[must_use]
    pub fn as_str(&self) -> &str {
        match self {
            Admission::Open => "open",
            Admission::List => "list",
            Admission::Other(text) => text,
        }
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for Admission {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Admission {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let text = <String as serde::Deserialize>::deserialize(deserializer)?;
        Ok(Admission::parse(&text))
    }
}

/// One entry of the `mediums` field of `/node/info` (ADR 0110 §8.1).
///
/// `group` is a plain string, so a reader does not fail on a group that this
/// release does not know.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct NodeMedium {
    /// The name of the group.
    pub group: String,
    /// The admission value of the group.
    pub admission: Admission,
}

/// The `mediums` of a node, or the default of ADR 0110 §8.2.
///
/// A node of an earlier release gives no `mediums`. A reader takes an absent
/// field as `[{"group": "music", "admission": "open"}]`.
#[must_use]
pub fn node_mediums_or_default(mediums: Option<Vec<NodeMedium>>) -> Vec<NodeMedium> {
    mediums.unwrap_or_else(|| {
        vec![NodeMedium {
            group: MediumGroup::Music.name().to_owned(),
            admission: Admission::Open,
        }]
    })
}
