use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[derive(Hash)]
pub enum HeadMoveDirection {
    Left = -1,
    Stay = 0,
    Right = 1
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TransitionSource {
    current_state: u64,
    current_symbol: u64
}
pub struct TransitionSourceBuilder {
    current_state: Option<u64>,
    current_symbol: Option<u64>
}
impl TransitionSourceBuilder {
    pub fn new() -> TransitionSourceBuilder {
        TransitionSourceBuilder {
            current_state: None,
            current_symbol: None
        }
    }
    pub fn set_current_state(&mut self, state: u64) {
        self.current_state = Some(state);
    }
    pub fn set_current_symbol(&mut self, symbol: u64) {
        self.current_symbol = Some(symbol);
    }
    pub fn build(&self) -> Option<TransitionSource> {
        if let (Some(state), Some(symbol)) =
            (self.current_state, self.current_symbol) {
            Some(TransitionSource {
                current_state: state,
                current_symbol: symbol
            })
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TransitionEffect {
    next_state: u64,
    symbol_to_write: u64,
    head_move_direction: HeadMoveDirection
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransitionRules {
    _rules: HashMap<TransitionSource, TransitionEffect>
}

pub struct TuringMachineDefinition {
    blank_symbol: u64,
    initial_tape: Vec<u64>,
    transition_rules: TransitionRules,
    start_state: u64,
    skip_state: u64,
}
impl TuringMachineDefinition {
    pub fn new(
        blank_symbol: u64,
        initial_tape: Vec<u64>,
        transition_rules: TransitionRules,
        start_state: u64,
        skip_state: u64,
    ) -> TuringMachineDefinition {
        TuringMachineDefinition {
            blank_symbol,
            initial_tape,
            transition_rules,
            start_state,
            skip_state,
        }
    }
    fn transition(
        &self,
        source: &TransitionSource
    ) -> Option<&TransitionEffect> {
        self.transition_rules._rules.get(source)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TuringMachineErrors {
    HeadMovedLeftOfTapeStart,
    InvalidTransition
}

pub struct OneEndedTuringMachine {
    definition: TuringMachineDefinition,
    tape: Vec<u64>,
    head_position: u64,
    current_state: u64
}
impl OneEndedTuringMachine {
    pub fn new(definition: TuringMachineDefinition) -> OneEndedTuringMachine {
        let tape = definition.initial_tape.clone();
        let start_state = definition.start_state;

        OneEndedTuringMachine {
            definition,
            tape,
            head_position: 0,
            current_state: start_state
        }
    }

    pub fn get_transition(&self) -> Result<TransitionEffect, TuringMachineErrors> {
        let current_symbol = self.tape[self.head_position as usize];
        let transition_source = TransitionSource {
            current_state: self.current_state,
            current_symbol
        };

        if let Some(effect) = self.definition.transition(&transition_source) {
            Ok(*effect)
        } else {
            Err(TuringMachineErrors::InvalidTransition)
        }
    }

    pub fn step(&mut self) -> Result<(), TuringMachineErrors> {
        let head_position = self.head_position as usize;
        let effect = self.get_transition()?;
        self.tape[head_position] = effect.symbol_to_write;

        match effect.head_move_direction {
            HeadMoveDirection::Left => {
                if self.head_position == 0 {
                    return Err(TuringMachineErrors::HeadMovedLeftOfTapeStart);
                }
                self.head_position -= 1;
            },
            HeadMoveDirection::Right => {
                self.head_position += 1;
                if self.head_position as usize >= self.tape.len() {
                    self.tape.push(self.definition.blank_symbol);
                }
            },
            HeadMoveDirection::Stay => {}
        }
        self.current_state = effect.next_state;
        Ok(())
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
