use crate::prelude::*;

use quick_xml::events::attributes::Attribute;
use quick_xml::events::BytesEnd;
use quick_xml::events::BytesStart;
use quick_xml::events::BytesText;
use quick_xml::events::Event;
use quick_xml::events;
use quick_xml::name::QName;
use quick_xml::reader::Reader;

use super::error::ParsedNameOrBytes;

use std::fmt;

#[derive(From, Debug)]
pub(crate) struct EventSummary {
    name: Option<ParsedNameOrBytes>,
    e_type: &'static str
}

impl fmt::Display for EventSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.name {
            Some(name) => write!(f, "element {name} with type {}", self.e_type),
            None => write!(f, "unnamed name with type {}", self.e_type),
        }
    }
}

impl EventSummary {
    pub(crate) fn new(e: &Event) -> Self {
        Self {
            name: e.event_name(),
            e_type: event_type(e),
        }
    }

    pub(crate) fn eof() -> Self {
        Self {
            name: None,
            e_type: "eof"
        }
    }

    pub(crate) fn start(bytes: &BytesStart<'_>) -> Self {
        Self {
            name: bytes.event_name(),
            e_type: "start"
        }
    }

    pub(crate) fn end(bytes: &BytesEnd<'_>) -> Self {
        Self {
            name: bytes.event_name(),
            e_type: "end"
        }
    }

    pub(crate) fn text(bytes: &BytesText<'_>) -> Self {
        Self {
            name: None,
            e_type: "text"
        }
    }
}

pub(crate) trait ElementName {
    fn event_name(&self) -> Option<ParsedNameOrBytes>;
    fn byte_name(&self) -> Option<QName<'_>>;
}

impl ElementName for BytesStart<'_> {
    fn event_name(&self) -> Option<ParsedNameOrBytes> {
        Some(ParsedNameOrBytes::from(self.name()))
    }

    fn byte_name(&self) -> Option<QName<'_>> {
        self.name().into()
    }
}

impl ElementName for BytesEnd<'_> {
    fn event_name(&self) -> Option<ParsedNameOrBytes> {
        Some(ParsedNameOrBytes::from(self.name()))
    }

    fn byte_name(&self) -> Option<QName<'_>> {
        self.name().into()
    }
}

impl ElementName for BytesText<'_> {
    fn event_name(&self) -> Option<ParsedNameOrBytes> {
        None
    }

    fn byte_name(&self) -> Option<QName<'_>> {
        None
    }
}

impl ElementName for events::BytesCData<'_> {
    fn event_name(&self) -> Option<ParsedNameOrBytes> {
        None
    }

    fn byte_name(&self) -> Option<QName<'_>> {
        None
    }
}

impl ElementName for events::BytesDecl<'_> {
    fn event_name(&self) -> Option<ParsedNameOrBytes> {
        None
    }

    fn byte_name(&self) -> Option<QName<'_>> {
        None
    }
}

impl ElementName for Event<'_> { 
    fn event_name(&self) -> Option<ParsedNameOrBytes> {
        ElementName::byte_name(self).map(|name| ParsedNameOrBytes::from(name))
    }

    fn byte_name(&self) -> Option<QName<'_>> {
        match &self {
            Event::Start(s) => s.byte_name(),
            Event::End(e) => e.byte_name(),
            Event::Empty(s) => s.byte_name(),
            Event::Text(x) => x.byte_name(),
            Event::Comment(x) => x.byte_name(),
            Event::CData(x) => x.byte_name(),
            Event::Decl(x) => x.byte_name(),
            Event::PI(x) => x.byte_name(),
            Event::DocType(x) => x.byte_name(),
            Event::Eof => None
        }
    }
}

fn event_type(event: &Event) -> &'static str {
    match event {
        Event::Start(_) => "start",
        Event::End(_) => "end",
        Event::Empty(_) => "empty",
        Event::Text(_) => "text",
        Event::Comment(_) => "comment",
        Event::CData(_) => "cdata",
        Event::Decl(_) => "decl",
        Event::PI(_) => "pi",
        Event::DocType(_) => "doctype",
        Event::Eof => "eof",
    }
}
