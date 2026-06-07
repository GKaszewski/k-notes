use domain::{events::DomainEvent, note::entity::NoteId, user::entity::UserId};

use crate::EventPayload;

fn note_created() -> DomainEvent {
    DomainEvent::NoteCreated {
        note_id: NoteId::new(),
        user_id: UserId::new(),
    }
}

#[test]
fn domain_event_round_trips_through_payload() {
    let event = note_created();
    let payload = EventPayload::from(&event);
    let recovered = DomainEvent::try_from(payload).unwrap();

    // Compare by serialising both — DomainEvent doesn't implement PartialEq.
    let EventPayload::NoteCreated {
        note_id: orig_nid,
        user_id: orig_uid,
    } = EventPayload::from(&event)
    else {
        panic!("wrong variant");
    };
    let EventPayload::NoteCreated {
        note_id: rec_nid,
        user_id: rec_uid,
    } = EventPayload::from(&recovered)
    else {
        panic!("wrong variant");
    };
    assert_eq!(orig_nid, rec_nid);
    assert_eq!(orig_uid, rec_uid);
}

#[test]
fn payload_serialises_to_json_and_back() {
    let event = note_created();
    let payload = EventPayload::from(&event);
    let bytes = payload.to_json().unwrap();
    let recovered = EventPayload::from_json(&bytes).unwrap();
    assert_eq!(payload, recovered);
}

#[test]
fn event_type_label_is_correct() {
    let uid = UserId::new();
    let nid = NoteId::new();
    assert_eq!(
        EventPayload::NoteCreated {
            note_id: nid.to_string(),
            user_id: uid.to_string()
        }
        .event_type(),
        "NoteCreated"
    );
    assert_eq!(
        EventPayload::NoteUpdated {
            note_id: nid.to_string(),
            user_id: uid.to_string()
        }
        .event_type(),
        "NoteUpdated"
    );
    assert_eq!(
        EventPayload::NoteDeleted {
            note_id: nid.to_string(),
            user_id: uid.to_string()
        }
        .event_type(),
        "NoteDeleted"
    );
}

#[test]
fn invalid_json_returns_error() {
    assert!(EventPayload::from_json(b"not json at all").is_err());
}

#[test]
fn invalid_uuid_in_payload_returns_error() {
    let payload = EventPayload::NoteCreated {
        note_id: "not-a-uuid".into(),
        user_id: "also-not-a-uuid".into(),
    };
    assert!(DomainEvent::try_from(payload).is_err());
}
