import random
import unittest
import tqdm

from automata_builder.firing_squad_automata import (
    FiringSquadAutomataRunner, F
)


class TestHalfCounterAutomataConvergence(unittest.TestCase):
    """
    The half-reducer automata is designed to transform t unary data cells
    into encoded cells with a value of (x+1)//2 in base n and in 2*t time,
    and such that the encoded n-ary is contained with the same
    position range as the original data cells

    (and in fact the left end of the encoded n-ary is at the same
    position as the left end of the original unary data cell range)
    """
    @staticmethod
    def _run_steps_and_read(
        write_start: int,
        write_end: int,
        steps: int,
    ) -> list[int]:
        runner = FiringSquadAutomataRunner(
            initial_write_start=write_start,
            initial_write_end=write_end,
        )
        runner.run_simulation(num_timesteps=steps, render=False)
        return runner.get_minimal_data_region()

    def test_firing_squad_synchronizes(
        self, num_tests: int = 100, seed: int = 67
    ) -> None:
        random.seed(seed)
        pbar = tqdm.tqdm(range(num_tests))

        for _ in pbar:
            write_start = random.choice(range(-100, 100))
            write_end = write_start + random.choice(range(0, 50))
            pbar.set_description(
                f'{write_start=} {write_end=} '
            )

            cells_filled = write_end - write_start + 1
            steps = cells_filled * 2

            with self.subTest(
                write_start=write_start,
                write_end=write_end,
                cells_filled=cells_filled,
                steps=steps,
            ):
                data_region = self._run_steps_and_read(
                    write_start=write_start,
                    write_end=write_end,
                    steps=steps,
                )
                expected_data_region = [F] * cells_filled

                self.assertEqual(
                    data_region,
                    expected_data_region,
                    msg=(
                        f"Expected data region {data_region=} "
                        f"after {steps} steps, got {expected_data_region=}."
                    ),
                )


if __name__ == "__main__":
    unittest.main()
