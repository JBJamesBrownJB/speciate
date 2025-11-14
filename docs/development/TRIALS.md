Testing technique where we run with pre-templated scenarios for visual and future automated testing.

An example trial would be:

Title: Crit weaving its way through a crowd
- Spawn a grid of obstacles (Crits in catatonic state) with set spacing
- Spawn a crit with certain properties, size, speed etc.. in 'seeking' state with its 'target' the other side of the grid of obstacles
- The grid should have the space between them set to smaller than the crits 'comfort' zone.
- Result, we should observe the crit successfully navigate its way through the grid and out the other side, without colliding with obscales.