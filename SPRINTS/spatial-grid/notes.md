
6. Crits use pseudo random neighbour sorting O(1).

7. Gain to only query neighbour cells if with perception range of cell wall?
What about FOV, can we quickly with math rule out entire cells based on position and orientatin within a cell?

8. In spatial-grid. Don't pull in anything that has zero acceleration / movement or can't move somehow.

9. In spatial-grid, dont rebuild the grid every tick, instead could we break it into 20 batches where over 20 ticks the whole grid is rebuilt? Maybe not a good optimisation?