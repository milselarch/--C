use std::collections::HashMap;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Direction {
    Left,
    Right,
    Middle,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct WritableTapeKey {
    tape_index: usize,
}
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct ReadableTapeKey {
    tape_index: usize,
}
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum TapeKey {
    Readable(ReadableTapeKey),
    Writable(WritableTapeKey),
}
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct TapeState {
    tape_key: TapeKey,
    tape_cell_state: u32
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum TapeCellState {
    VOID,
    TapeState(TapeState),
    HALT,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct CellExpectation {
    direction: Direction,
    tape_key: TapeKey,
}
#[derive(Debug, Clone)]
pub struct WriteRule {
    expectations: Vec<CellExpectation>,
    write_tape: WritableTapeKey,
    write_value: TapeCellState,
}


#[derive(Debug, Clone)]
pub struct Tape {
    // whether cellular automata rules applied to cells on
    // this tape can write to the tape itself
    self_writeable: bool,
    write_rules: Vec<WriteRule>,
    data: Vec<u32>,
}
impl Tape {
    pub fn new(
        self_writeable: bool,
        write_rules: Vec<WriteRule>,
        data: Vec<u32>,
    ) -> Tape {
        if !self_writeable {
            assert_eq!(
                write_rules.len(), 0,
                "Non-self-writeable tapes cannot have write rules"
            );
        }
        Tape {
            self_writeable,
            write_rules,
            data,
        }
    }
}

#[derive(Debug, Clone)]
pub struct MultiTape {
    read_tapes: Vec<Tape>,
    write_tapes: Vec<Tape>,
    tape_names_map: HashMap<String, TapeKey>,
}
impl MultiTape {
    pub fn new(
        read_tapes: Vec<Tape>,
        write_tapes: Vec<Tape>,
    ) -> MultiTape {
        MultiTape {
            read_tapes,
            write_tapes,
            tape_names_map: Default::default(),
        }
    }
    pub fn get_tape_key(&self, name: &str) -> Option<&TapeKey> {
        self.tape_names_map.get(name)
    }
    pub fn insert_named_tape(
        &mut self, name: String, tape: Tape
    ) -> TapeKey{
        let tape_key = if tape.self_writeable {
            let tape_index = self.write_tapes.len();
            self.write_tapes.push(tape);
            TapeKey::Writable(WritableTapeKey { tape_index })
        } else {
            let tape_index = self.read_tapes.len();
            self.read_tapes.push(tape);
            TapeKey::Readable(ReadableTapeKey { tape_index })
        };

        self.tape_names_map.insert(name, tape_key.clone());
        return tape_key;
    }
}
