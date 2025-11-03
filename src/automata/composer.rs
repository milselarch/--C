use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use enum_iterator::Sequence;

/*
TODO:
    - add rules accumulator
    - translate rules to multi-tape equations
    - compose multi-tape automata to single-tape automata
    - APL style rule builders (accumulator, reduct input, unary expansion)
*/

#[derive(Debug, Clone, Eq, PartialEq, Hash, Ord, PartialOrd, Sequence)]
pub enum Direction {
    Left,
    Right,
    Middle,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct WritableTapeKey {
    tape_index: usize,
}
#[derive(Clone, Debug, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct ReadableTapeKey {
    tape_index: usize,
}
#[derive(Clone, Debug, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub enum TapeKey {
    Readable(ReadableTapeKey),
    Writable(WritableTapeKey),
}
#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub struct TapeState {
    tape_key: TapeKey,
    tape_cell_state: u32
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum TapeCellState {
    VOID,
    HALT,
    TapeState(TapeState),
}
impl Ord for TapeCellState {
    fn cmp(&self, other: &Self) -> Ordering {
        use TapeCellState::*;
        match (self, other) {
            (VOID, VOID) => Ordering::Equal,
            (VOID, _) => Ordering::Less,
            (_, VOID) => Ordering::Greater,
            (HALT, HALT) => Ordering::Equal,
            (HALT, _) => Ordering::Greater,
            (_, HALT) => Ordering::Less,
            (TapeState(a), TapeState(b)) => a.cmp(b),
        }
    }
}
impl PartialOrd for TapeCellState {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct CellExpectation {
    direction: Direction,
    expected_state: TapeCellState,
}
pub fn new_cell_expectation(
    direction: Direction,
    expected_state: TapeCellState
) -> CellExpectation {
    CellExpectation {
        direction,
        expected_state,
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CellExpectationCombo {
    cell_expectations: HashSet<CellExpectation>,
}
impl CellExpectationCombo {
    pub fn new(
        cell_expectations: HashSet<CellExpectation>
    ) -> CellExpectationCombo {
        CellExpectationCombo { cell_expectations }
    }
    pub fn new_empty() -> CellExpectationCombo {
        CellExpectationCombo {
            cell_expectations: HashSet::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct WriteRule {
    expectations: CellExpectationCombo,
    write_tape: WritableTapeKey,
    // cell transition to apply to own tapes cell
    self_write_value: TapeCellState,
    // cell transitions to apply to other read tape cells
    // at the same position
    read_tape_write_values: Vec<(ReadableTapeKey, TapeCellState)>,
}
impl Hash for CellExpectationCombo {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let mut expectations_vec: Vec<&CellExpectation> =
            self.cell_expectations.iter().collect();
        expectations_vec.sort();
        for expectation in expectations_vec {
            expectation.hash(state);
        }
    }
}


#[derive(Debug, Clone)]
pub struct Tape {
    // whether cellular automata rules applied to cells on
    // this tape can write to the tape itself
    self_writeable: bool,
    write_rules: Vec<WriteRule>,
    allowed_states: HashSet<u32>,
    // cells extending rightwards
    data: Vec<u32>,
    // cells extending leftwards
    rev_data: Vec<u32>
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
            allowed_states: Default::default(),
            data,
            rev_data: vec![],
        }
    }
    pub fn generate_all_combinations(&self) -> HashSet<CellExpectationCombo> {
        /*
        Generates all possible combinations of cells within a 1-cell radius
        */
        let mut combinations = HashSet::new();

        for direction in enum_iterator::all::<Direction>() {
            let mut combination = CellExpectationCombo::new_empty();
            for state in &self.allowed_states {
                todo!()
            }
        }
        todo!()
    }
}

#[derive(Debug, Clone)]
pub struct MultiTape {
    read_tapes: Vec<Tape>,
    write_tapes: Vec<Tape>,
    tape_names_map: HashMap<String, TapeKey>,
    rules: Vec<WriteRule>,
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
            rules: vec![],
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
    pub fn generate_tape_equations(&self) {
        let mut equations = HashMap::new();
        for rule in &self.rules {

        }
    }
}
