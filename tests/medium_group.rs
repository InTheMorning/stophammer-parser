use stophammer_parser::medium_group::{
    Admission, MediumClass, MediumGroup, NodeMedium, classify, node_mediums_or_default,
};

#[test]
fn classify_gives_the_group_of_each_medium() {
    for text in ["music", "MUSIC", " musicL ", "musicl"] {
        assert_eq!(
            classify(Some(text)),
            MediumClass::Group(MediumGroup::Music),
            "{text:?}"
        );
    }
    for text in ["podcast", "podcastL"] {
        assert_eq!(
            classify(Some(text)),
            MediumClass::Group(MediumGroup::Podcast),
            "{text:?}"
        );
    }
    assert_eq!(classify(Some("Publisher")), MediumClass::Publisher);
    for text in ["video", "film", "audiobook"] {
        assert_eq!(classify(Some(text)), MediumClass::NoGroup, "{text:?}");
    }
    assert_eq!(classify(None), MediumClass::Absent);
    assert_eq!(classify(Some("")), MediumClass::Absent);
}

#[test]
fn each_group_has_its_default_medium() {
    assert_eq!(MediumGroup::Music.default_medium(), "music");
    assert_eq!(MediumGroup::Podcast.default_medium(), "podcast");
    assert_eq!(MediumGroup::from_name("music"), Some(MediumGroup::Music));
    assert_eq!(MediumGroup::from_name("Music"), None);
    assert_eq!(MediumGroup::from_name("video"), None);
    for group in MediumGroup::ALL {
        assert_eq!(MediumGroup::from_name(group.name()), Some(group));
    }
}

#[cfg(feature = "serde")]
#[test]
fn an_unknown_admission_reads_as_other() {
    let json = r#"{"group":"podcast","admission":"rule"}"#;
    let medium: NodeMedium = serde_json::from_str(json).unwrap();
    assert_eq!(medium.admission, Admission::Other("rule".to_owned()));
    assert_eq!(serde_json::to_string(&medium).unwrap(), json);
    assert_eq!(Admission::parse("open"), Admission::Open);
    assert_eq!(Admission::parse("list"), Admission::List);
}

#[test]
fn an_absent_mediums_field_is_music_open() {
    let mediums = node_mediums_or_default(None);
    assert_eq!(
        mediums,
        vec![NodeMedium {
            group: "music".to_owned(),
            admission: Admission::Open,
        }]
    );
}
