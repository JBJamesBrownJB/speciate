Don't render crits off screen.

Don't even add them to the NAPI buffer...!?!?!

More sophisticated

- Cull / don't send off screen crits to NAPI at all
- Above a set zoom level, just render pixel? when zoomed out, yhou can't even make out the shape of the sprite anyway? have some sort of visual LOD system?
- Maybe at a certain zoom, just show regional indication of crit existance?

- Below zoome of 2.0 stop using sprites and just use pixels, reduce NAPI buffer data to juse pos x,y

Any other perf boosts?