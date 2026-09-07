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

        self.counter_automata_runner = CounterAutomataRunner(
            base=self.base,
            initial_write_start=self.initial_write_start,
            initial_write_end=self.initial_write_end,
            apply_reduction=self.apply_reduction
        )