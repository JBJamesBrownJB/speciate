## Interger overflows
I suspect, due to rapdid nature of everything that anywhere that has ever increasing integers will face integer overflows. We need to do a comprehensive sweep of where these will occur and come up with a plan of action.

- ticks
- CritId
- ...

## NanoIds or uuids
For things like agent ids and species ids (DNA that can replicate any agent) should use nanoID. Maybe for any unique thing in the world, NanoId should be used?

## Missing Portal Error state
Currently the portal shows 'Reconnectin' error state (and orange glow backlight) when it can not communicate with the broadcaster websocket. However, if the simuation is not up or unable to communicate with broadcaster (maybe NAT server down) then this is also a error state. So currently the portal shows green glow (signalling working) but there are not crits. 

## Crits never give up on a target
At the moment, a crit seeking a target will just kind of get stuck trying to get there even if its already occupied by a crit or something else. We need a circuit breaker in its behaviour, such that if it can't get there after a few attempts, it should change behaviour or pick new target or something.