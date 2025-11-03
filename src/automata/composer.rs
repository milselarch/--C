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

const VOID_STATE: u32 = 0;

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
impl TapeState {
    pub fn new(
        tape_key: TapeKey,
        tape_cell_state: u32
    ) -> TapeState {
        TapeState {
            tape_key,
            tape_cell_state,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct CellExpectation {
    direction: Direction,
    expected_state: TapeState,
}
impl CellExpectation {
    pub fn new(
        direction: Direction,
        expected_state: TapeState
    ) -> CellExpectation {
        CellExpectation {
            direction,
            expected_state,
        }
    }

    pub fn to_identifier(&self) -> TapeCellIdentifier {
        TapeCellIdentifier::new(
            self.expected_state.tape_key.clone(),
            self.direction.clone(),
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct TapeCellIdentifier {
    tape_key: TapeKey,
    direction: Direction,
}
impl TapeCellIdentifier {
    pub fn new(
        tape_key: TapeKey,
        direction: Direction
    ) -> TapeCellIdentifier {
        TapeCellIdentifier {
            tape_key,
            direction,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CellExpectationCombo {
    /*
    Represents the expectation that a bunch of adjacent cells
    certain corresponding states
    */
    cell_expectations: HashMap<TapeCellIdentifier, CellExpectation>
}
impl CellExpectationCombo {
    pub fn new(
        cell_expectations: HashMap<TapeCellIdentifier, CellExpectation>
    ) -> CellExpectationCombo {
        CellExpectationCombo { cell_expectations }
    }
    pub fn new_empty() -> CellExpectationCombo {
        CellExpectationCombo {
            cell_expectations: HashMap::new()
        }
    }
    pub fn insert_expectation(
        &mut self, expectation: CellExpectation
    ) {
        let identifier = expectation.to_identifier();
        // ensure no duplicate expectations for same tape cell
        let prev_value = self.cell_expectations.insert(identifier, expectation);
        assert_eq!(prev_value, None);
    }
}

#[derive(Debug, Clone)]
pub struct WriteRule {
    expectations: CellExpectationCombo,
    write_tape: WritableTapeKey,
    // cell transition to apply to own tapes cell
    self_write_value: TapeState,
    // cell transitions to apply to other read tape cells
    // at the same position
    read_tape_write_values: Vec<(ReadableTapeKey, TapeState)>,
}
impl Hash for CellExpectationCombo {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let mut expectations_vec: Vec<&CellExpectation> =
            self.cell_expectations.values().collect();
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
    tape_index: usize,

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
        tape_index: usize,
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
            tape_index,
            data,
            rev_data: vec![],
        }
    }
    pub fn build_cell_expectation(
        &self, tape_cell_state: u32, direction: Direction
    ) -> CellExpectation {
        let tape_state = TapeState::new(
            if self.self_writeable {
                TapeKey::Writable(WritableTapeKey {
                    tape_index: self.tape_index,
                })
            } else {
                TapeKey::Readable(ReadableTapeKey {
                    tape_index: self.tape_index,
                })
            },
            tape_cell_state,
        );
        let cell_expectation = CellExpectation::new(
            direction,
            tape_state,
        );
        cell_expectation
    }
    pub fn generate_all_combinations(&self) -> HashSet<CellExpectationCombo> {
        /*
        Generates all possible combinations of cells within a 1-cell radius
        */
        let mut combinations = HashSet::new();

        for direction in enum_iterator::all::<Direction>() {
            let mut combination = CellExpectationCombo::new_empty();
            for tape_cell_state in &self.allowed_states {
                let cell_expectation = self.build_cell_expectation(
                    *tape_cell_state, direction.clone()
                );
                combination.insert_expectation(cell_expectation);
            }

            combinations.insert(combination);
        }
        combinations
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
        for (index, tape) in read_tapes.iter().enumerate() {
            assert_eq!(
                tape.self_writeable, false,
                "Read tape at index {} is self-writeable",
                index
            );
            assert_eq!(tape.tape_index, index);
        }
        for (index, tape) in write_tapes.iter().enumerate() {
            assert_eq!(
                tape.self_writeable, true,
                "Write tape at index {} is not self-writeable",
                index
            );
            assert_eq!(tape.tape_index, index);
        }

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
        todo!()
    }
}
