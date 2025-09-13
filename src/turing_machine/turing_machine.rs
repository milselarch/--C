pub struct TuringMachineDefinition {
    blank_symbol: u64,
    initial_tape: Vec<u64>,
    start_state: u64,
    skip_state: u64,
    halt_state: u64
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeadMoveDirection {
    Left = -1,
    Stay = 0,
    Right = 1
}

pub trait TuringMachineRuleset: Sized {
    fn get_definition(&self) -> &TuringMachineDefinition;
    // TODO: how guarantee this is deterministic?
    // Returns (next_state, symbol_to_write, head_move_direction)
    fn get_next_state(
        &self, current_state: u64, current_symbol: u64
    ) -> Option<(u64, u64, HeadMoveDirection)>;
}

pub struct OneEndedTuringMachine {
    definition: TuringMachineRuleset,
    tape: Vec<u64>,
    head_position: u64,
    current_state: u64
}
impl OneEndedTuringMachine {
    pub fn new(definition: TuringMachineDefinition) -> OneEndedTuringMachine {
        OneEndedTuringMachine {
            definition,
            tape: definition.initial_tape.clone(),
            head_position: 0,
            current_state: definition.start_state
        }
    }

    pub fn step(&mut self) {
        todo!()
    }

    pub fn run(&mut self) {
        while self.current_state != self.definition.halt_state {
            self.step();
        }
    }

    pub fn get_tape(&self) -> &Vec<u64> {
        &self.tape
    }

    pub fn get_head_position(&self) -> u64 {
        self.head_position
    }

    pub fn get_current_state(&self) -> u64 {
        self.current_state
    }
}
