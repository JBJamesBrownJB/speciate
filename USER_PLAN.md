 Remove the neighbour count perception trigger reduction per crit and instead keep a total track of full neighbour counts and trigger global skipping at threshold.

So we start with fast every tick, then it can circuit break back to up to skipping 4 ticks.

Need to make sure crits modulus I'd is 4 then.

Step 1: Research the code and summarise the current percetption tick state, I think we hvae one global skip based on a modulus id on crits sot that each crit only runs perception on tick n, and there is another system that when neighbours fill up, that crit will skip n ticks.

