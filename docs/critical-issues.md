## Interger overflows
I suspect, due to rapdid nature of everything that anywhere that has ever increasing integers will face integer overflows. We need to do a comprehensive sweep of where these will occur and come up with a plan of action.

- tick / ticks
- CritId
- ...

## Snapshots load crits but not moving
I tested loading from a snapshot, all crits were there but all stationary. Maybe something is not part of the snapshot or loaded properly. Maybe behaviour?

Also, world bounds looked off, maybe snapshot is not serialising/deserialising that correctly as well. How do we ensure snapshot logic keeps up to date with our changes!??!?

## NanoIds or uuids
For things like agent ids and species ids (DNA that can replicate any agent) should use nanoID. Maybe for any unique thing in the world, NanoId should be used?

## IPC Error Handling
The portal (frontend) should detect when the Electron IPC connection fails or the simulation subprocess crashes. Currently there's no visual feedback when state updates stop arriving (simulation not running). Need error state UI showing:
- "Simulation subprocess not responding"
- "Electron IPC connection lost"
- "No state updates received for >5 seconds"

## Crits never give up on a target
At the moment, a crit seeking a target will just kind of get stuck trying to get there even if its already occupied by a crit or something else. We need a circuit breaker in its behaviour, such that if it can't get there after a few attempts, it should change behaviour or pick new target or something.