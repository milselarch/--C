from typing import Final

from automata_builder.rule_generator import TapeCellState

X: Final[TapeCellState] = TapeCellState(0)  # void state
H: Final[TapeCellState] = TapeCellState(1)  # invalid / halt state

L: Final[TapeCellState] = TapeCellState(2)  # initial data state
A: Final[TapeCellState] = TapeCellState(3)
B: Final[TapeCellState] = TapeCellState(4)
C: Final[TapeCellState] = TapeCellState(5)
G: Final[TapeCellState] = TapeCellState(6)  # general state
F: Final[TapeCellState] = TapeCellState(7)  # firing state

RULE_MATRICES: dict[int, dict[int, dict[int, int]]] = {
    L: {
        X: {X: H, L: L, A: H, B: H, C: H, G: H},
        L: {X: L, L: L, A: H, B: L, C: L, G: L},
        A: {X: C, L: G, A: L, B: L, C: L, G: C},
        B: {X: L, L: L, A: L, B: L, C: L, G: L},
        C: {X: G, L: A, A: L, B: L, C: L, G: G},
        G: {X: A, L: C, A: L, B: L, C: L, G: A},
    },
    A: {
        X: {X: H, L: H, A: F, B: H, C: G, G: H},
        L: {X: H, L: H, A: A, B: L, C: G, G: H},
        A: {X: F, L: A, A: A, B: B, C: C, G: B},
        B: {X: C, L: G, A: H, B: G, C: C, G: C},
        C: {X: H, L: A, A: A, B: H, C: H, G: H},
        G: {X: C, L: H, A: H, B: H, C: C, G: C},
    },
    B: {
        X: {X: H, L: H, A: H, B: H, C: H, G: H},
        L: {X: H, L: H, A: G, B: B, C: L, G: B},
        A: {X: H, L: G, A: B, B: B, C: L, G: H},
        B: {X: H, L: G, A: A, B: B, C: C, G: B},
        C: {X: L, L: L, A: A, B: H, C: H, G: L},
        G: {X: G, L: C, A: C, B: H, C: B, G: G},
    },
    C: {
        X: {X: H, L: H, A: H, B: H, C: H, G: H},
        L: {X: H, L: C, A: A, B: G, C: C, G: G},
        A: {X: B, L: B, A: H, B: B, C: H, G: B},
        B: {X: G, L: C, A: H, B: H, C: C, G: G},
        C: {X: H, L: C, A: A, B: B, C: C, G: B},
        G: {X: B, L: B, A: H, B: B, C: H, G: B},
    },
    G: {
        X: {X: H, L: A, A: H, B: G, C: G, G: F},
        L: {X: H, L: H, A: G, B: G, C: G, G: H},
        A: {X: H, L: B, A: H, B: G, C: G, G: H},
        B: {X: G, L: B, A: H, B: G, C: G, G: G},
        C: {X: A, L: A, A: H, B: G, C: G, G: A},
        G: {X: F, L: B, A: H, B: G, C: G, G: F},
    }
}
