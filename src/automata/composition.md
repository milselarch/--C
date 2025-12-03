Plan to compose a multi-tape automata into a single-tape automata

1. pre-requisites
    1. we have n read-only tapes and m write-only tapes
    2. The input value has scoped as a predefined set of states that 
       go into one of the input read-only tapes
    3. There can never be a gap in the input read-only tape
       i.e. the input read-only tape must always contain exactly 1 contiguous
       block of non-void cells, and the rest are void cells
    4. cells in the read-only tape cannot transition unless there 
       is a rule involving non-void cells in the write tape
    5. non-void cells in a write-tape can cause a 
       transition to themselves and read-only tapes
    6. a write-only tape cannot write to another write-only tape
    7. a read tape can only be written to by at most one write tape
2. generation of rules for the multi-tape cellular automata
    1. for every write tape W that writes to a read tape R
        1. for every possible combination of cells CW in W 
           that *does not* write to the read tape
            1. multiply that with every combination of read tape cells 
               CR and add that (CR\*CW) to the equations of the read-only tape
        2. for every possible combination of cells CW in W 
           that *does* write to the read tape
            1. just add that the combination to the 
               equations for the read-only tape
3. folding of multi-tape automata to a single-tape automata
    1. in the single tape automata the states that exist 
       will simply be the product of all the possible states in each 
       of the individual multi-tape automatas
    2. the equations for each state for each tape in the 
       multi-tape automata can simply be remapped to be the 
       equations for the corresponding state in the single-tape 
       automata - of course dependent states in the equations will 
       also have to be remapped, but structurally the equations will be the same.
4. Neighbors map generation
   1. Start with the input tape
   2. We consider all combinations of VOID and input states