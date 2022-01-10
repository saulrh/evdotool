use evdev_rs::enums::{EventCode, EventType};
use evdev_rs::util::EventCodeIterator;
use lazy_static::lazy_static;
use std::collections::HashMap;

lazy_static! {
    static ref MIN_CODES: HashMap<EventType, EventCode> = {
        let mut m = HashMap::new();
        for event_type in MIN_TYPE.iter() {
            if let Some(max_code_raw) = EventType::get_max(&event_type) {
                let min_code = (0u32..=max_code_raw as u32)
                    .map(|raw_event_code| {
                        evdev_rs::util::int_to_event_code(event_type as u32, raw_event_code)
                    })
                    .filter_map(|ec| match ec {
                        EventCode::EV_UNK { .. } => None,
                        other => Some(other),
                    })
                    .next()
                    .unwrap();
                m.insert(event_type, min_code);
            }
        }
        m
    };
}

const MIN_TYPE: EventType = EventType::EV_SYN;

pub fn all_event_codes() -> EventCodeIterator {
    EventCode::iter(&MIN_CODES[&MIN_TYPE])
}

pub struct CodesInTypeIter {
    internal: EventCodeIterator,
    ev_type: EventType,
}

impl Iterator for CodesInTypeIter {
    type Item = EventCode;

    fn next(&mut self) -> Option<Self::Item> {
        let max_code_raw = EventType::get_max(&self.ev_type).unwrap();
        match self.internal.next() {
            None => None,
            Some(next_code) => {
                let (_, ec_raw) = evdev_rs::util::event_code_to_int(&next_code);
                if ec_raw == max_code_raw as u32 {
                    None
                } else {
                    Some(next_code)
                }
            }
        }
    }
}

pub fn codes_for(ev_type: EventType) -> Option<CodesInTypeIter> {
    MIN_CODES.get(&ev_type).map(|min_code| CodesInTypeIter {
        ev_type,
        internal: min_code.iter(),
    })
}

#[derive(thiserror::Error, Debug)]
pub enum CodeFromStrError {
    #[error("could not split `{0}` into <type>_<code> (e.g. KEY_SPACE)")]
    NoSeparator(String),
    #[error("unknown event type `{0}`")]
    UnknownEventType(String),
    #[error("unknown event code `{0}` for event type `{1}`")]
    UnknownEventCode(String, EventType),
}

pub fn event_code_from_str(s: String) -> Result<EventCode, CodeFromStrError> {
    let (type_prefix, _) = s
        .split_once("_")
        .ok_or_else(|| CodeFromStrError::NoSeparator(String::from(&s)))?;
    let type_name = String::from("EV_") + type_of_event_code(&type_prefix);
    let event_type = EventType::from_str(&type_name)
        .ok_or_else(|| CodeFromStrError::UnknownEventType(String::from(type_name)))?;
    let result = EventCode::from_str(&event_type, &s)
        .ok_or_else(|| CodeFromStrError::UnknownEventCode(String::from(&s), event_type))?;
    Ok(result)
}

pub fn type_of_event_code(code: &str) -> &str {
    if code == "BTN" {
        "KEY"
    } else {
        code
    }
}
