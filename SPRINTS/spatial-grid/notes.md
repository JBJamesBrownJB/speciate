8. In spatial-grid. Don't pull in anything that has zero acceleration / movement or can't move somehow. Do via component elimination as in Without<Acceleration> or something?

9. In spatial-grid, dont rebuild the grid every tick, instead could we break it into 20 batches where over 20 ticks the whole grid is rebuilt? Maybe not a good optimisation? Or essentially batches = tick_time and then every 1 second, the whole grid will be rebuilt?

10. Adding ONLY catatonic crits increases movement system latency... make no sense!

11. Use topological neighbour sorting for crits near to the camera but fall back to pseudo random ones for ones away from camera?

12. Dynamic Range Reduction in Crowds, see docs / biology/ideas