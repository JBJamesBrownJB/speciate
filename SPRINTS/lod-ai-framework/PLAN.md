We want to introduce the concept of AI LOD (Level Of Distance).

The first use of this is we will have two seperate perception algorithms depending on the distance from the viewport in the portal.

e.g. Crits in view will use the current  and accurate topological sorting neighbour algorithm, but crits outside view will fall back to the less computationaly intensive pseudo-random O(1) algoirthm (still better than id based right?).

But this will also need to lay the foundations for more LOD-AI things in the future, so we need an elegant and simple strategy to enable this.

The system should allow for two dimensions of choosing LOD levels.

When camera is zoomed in by a certain amount 
- Viewport + x% buffer area will use 'high-def AI'. Outside will use low-def AI.

When camera is zoomed out by x% (viewing alot of small crtis)
- Alls systems move to low-def AI

The first thing we will do, is have the system mark cells that are low/hi-def and send these to the portal (feature flagged for --dev-tools) so we can visualise it in portal.

Some initial ideas for hi/low def AI:
- Neighbour sorting PRIORITY