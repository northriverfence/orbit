//! ANSI/VT100 escape sequence parser

use vte::{Params, Perform};

#[derive(Debug, Clone)]
pub enum ParsedEvent {
    Print(char),
    Execute(u8),
    CsiDispatch(Vec<i64>, Vec<u8>, bool, char),
    EscDispatch(Vec<u8>, bool, u8),
}

pub struct AnsiParser {
    performer: VtePerformer,
}

impl AnsiParser {
    pub fn new() -> Self {
        Self {
            performer: VtePerformer::new(),
        }
    }

    pub fn parse(&mut self, data: &[u8]) -> Vec<ParsedEvent> {
        let mut parser = vte::Parser::new();
        for byte in data {
            parser.advance(&mut self.performer, *byte);
        }
        self.performer.take_events()
    }
}

struct VtePerformer {
    events: Vec<ParsedEvent>,
}

impl VtePerformer {
    fn new() -> Self {
        Self { events: Vec::new() }
    }

    fn take_events(&mut self) -> Vec<ParsedEvent> {
        std::mem::take(&mut self.events)
    }
}

impl Perform for VtePerformer {
    fn print(&mut self, c: char) {
        self.events.push(ParsedEvent::Print(c));
    }

    fn execute(&mut self, byte: u8) {
        self.events.push(ParsedEvent::Execute(byte));
    }

    fn hook(&mut self, _params: &Params, _intermediates: &[u8], _ignore: bool, _c: char) {}

    fn put(&mut self, _byte: u8) {}

    fn unhook(&mut self) {}

    fn osc_dispatch(&mut self, _params: &[&[u8]], _bell_terminated: bool) {}

    fn csi_dispatch(&mut self, params: &Params, intermediates: &[u8], ignore: bool, c: char) {
        let params_vec: Vec<i64> = params
            .iter()
            .flat_map(|p| p.iter())
            .map(|&x| x as i64)
            .collect();
        self.events.push(ParsedEvent::CsiDispatch(
            params_vec,
            intermediates.to_vec(),
            ignore,
            c,
        ));
    }

    fn esc_dispatch(&mut self, intermediates: &[u8], ignore: bool, byte: u8) {
        self.events.push(ParsedEvent::EscDispatch(
            intermediates.to_vec(),
            ignore,
            byte,
        ));
    }
}
