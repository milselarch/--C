from automata_builder.counter_automata import (
    CounterAutomataRunner, DATA_TAPE, DT_DATA
)
from automata_builder.rule_generator_multitape import MultiTapeBuilder
from automata_builder.tape_overlaps import MultiTapeState

INPUT_DATA_STATE = MultiTapeState(tape_no=DATA_TAPE, tape_cell_state=DT_DATA)


class ComposedCounterAutomataRunner(object):
    def __init__(
        self, base: int = 8, initial_write_start: int = 0,
        initial_write_end: int = 20,
        apply_reduction: bool = False
    ):
        self.base = base
        self.initial_write_start = initial_write_start
        self.initial_write_end = initial_write_end
        self.apply_reduction = apply_reduction

        self.runner = CounterAutomataRunner(
            base=self.base,
            initial_write_start=self.initial_write_start,
            initial_write_end=self.initial_write_end,
            apply_reduction=self.apply_reduction
        )

        self.multi_tape_builder = MultiTapeBuilder(
            multi_tape_automata=self.runner.multi_tape_automata
        )
        self.multi_tape_builder.declare_initial_group_overlaps(
            overlap_states={INPUT_DATA_STATE}
        )
        self.compose_result = self.multi_tape_builder.compose_tapes()
