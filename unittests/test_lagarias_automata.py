import math
import unittest

from fractions import Fraction

from automata_builder.lagarias_automata import (
    CONTROL_TAPE,
    OUTPUT_HALT_STATE,
    OUTPUT_TRUE_STATE,
    OUTPUT_TAPE,
    LagariasAutomataBuilder,
    LagariasAutomataRunner,
    modulo_by_repeated_subtraction,
    sigma_via_divisor_scan,
)

try:
    from automata_builder.counter_automata import CounterAutomataRunner  # noqa: F401
    HAS_COUNTER_AUTOMATA = True
except ModuleNotFoundError:
    HAS_COUNTER_AUTOMATA = False


class TestLagariasAutomata(unittest.TestCase):
    @staticmethod
    def _transition_signature(transition) -> tuple:
        input_terms = []
        for term in transition.input_terms:
            if hasattr(term, "get_position"):
                position = term.get_position()
                tape_no = term.get_tape_no()
                state = term.get_state()
            else:
                position = term.position
                tape_no = term.tape_no
                state = term.state
            input_terms.append((int(position), int(tape_no), int(state)))

        if hasattr(transition, "output_state"):
            output_tape_no = int(transition.output_state.tape_no)
            output_cell_state = int(transition.output_state.tape_cell_state)
        else:
            output_tape_no = int(transition.output_tape_no)
            output_cell_state = int(transition.output_cell_state)

        return (
            tuple(sorted(input_terms)),
            output_tape_no,
            output_cell_state,
            transition.annotation,
        )

    def test_transition_group_families_exist(self) -> None:
        builder = LagariasAutomataBuilder(base=6)
        family_names = [group.name for group in builder.build_transitions_group()]
        self.assertEqual(
            family_names,
            [
                'SWEEP_CLOCK_BOUNDARY',
                'ARITHMETIC_KERNELS',
                'MODULO_REPEATED_SUBTRACTION',
                'SIGMA_DIVISOR_SUM_LOOP',
                'HARMONIC_INTERVAL_ACCUMULATION',
                'LOG_EXP_INTERVAL_SERIES',
                'RHS_INTERVAL_COMPOSITION',
                'DECISION_AND_HALT',
            ],
        )

    def test_modulo_by_repeated_subtraction(self) -> None:
        self.assertEqual(modulo_by_repeated_subtraction(0, 1), 0)
        self.assertEqual(modulo_by_repeated_subtraction(20, 6), 2)
        self.assertEqual(modulo_by_repeated_subtraction(20, 5), 0)
        self.assertEqual(modulo_by_repeated_subtraction(21, 7), 0)

    def test_n_equals_one_shortcut_sets_true_halt(self) -> None:
        runner = LagariasAutomataRunner(
            base=6,
            initial_write_start=0,
            initial_write_end=0,
            derive_n_via_counter_automata=False,
        )
        decision = runner.evaluate()
        self.assertTrue(decision.holds)
        self.assertTrue(decision.used_n_equals_one_shortcut)
        self.assertEqual(decision.output_state, OUTPUT_TRUE_STATE)
        self.assertEqual(
            runner.tape_registers[OUTPUT_TAPE],
            (OUTPUT_TRUE_STATE, OUTPUT_HALT_STATE),
        )

    def test_step_advances_control_via_transition_schedule(self) -> None:
        runner = LagariasAutomataRunner(
            base=6,
            initial_write_start=0,
            initial_write_end=4,
            derive_n_via_counter_automata=False,
        )
        self.assertGreater(len(runner.transitions_group), 0)
        initial_index = runner.current_snapshot_index
        initial_control = runner.tape_registers[CONTROL_TAPE]

        runner.step(verbose=False)

        self.assertEqual(runner.current_snapshot_index, initial_index + 1)
        self.assertNotEqual(
            runner.tape_registers[CONTROL_TAPE],
            initial_control,
        )

    def test_timestep_transition_rules_are_input_independent(self) -> None:
        runner_small = LagariasAutomataRunner(
            base=8,
            initial_write_start=0,
            initial_write_end=4,
            derive_n_via_counter_automata=False,
        )
        runner_large = LagariasAutomataRunner(
            base=8,
            initial_write_start=0,
            initial_write_end=14,
            derive_n_via_counter_automata=False,
        )

        small_signatures = [
            self._transition_signature(transition)
            for transition in runner_small.transitions_group.transitions
        ]
        large_signatures = [
            self._transition_signature(transition)
            for transition in runner_large.transitions_group.transitions
        ]
        self.assertEqual(small_signatures, large_signatures)

    @unittest.skipUnless(
        HAS_COUNTER_AUTOMATA,
        "Counter automata dependency (py_ca_compiler) is unavailable.",
    )
    def test_counter_automata_derives_unary_n(self) -> None:
        runner = LagariasAutomataRunner(
            base=7,
            initial_write_start=-4,
            initial_write_end=4,
            derive_n_via_counter_automata=True,
        )
        self.assertEqual(runner.n_value, 9)

    def test_matches_direct_numeric_reference_small_range(self) -> None:
        for n in range(2, 41):
            with self.subTest(n=n):
                runner = LagariasAutomataRunner(
                    base=8,
                    initial_write_start=0,
                    initial_write_end=n - 1,
                    derive_n_via_counter_automata=False,
                )
                decision = runner.evaluate()
                harmonic = sum(1.0 / k for k in range(1, n + 1))
                rhs = harmonic + math.exp(harmonic) * math.log(harmonic)
                expected = sigma_via_divisor_scan(n) <= rhs
                self.assertEqual(decision.holds, expected)

                sigma_fraction = Fraction(decision.sigma_n, 1)
                self.assertLessEqual(sigma_fraction, decision.rhs_interval.upper)


if __name__ == "__main__":
    unittest.main()
