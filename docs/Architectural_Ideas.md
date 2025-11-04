## Decouple websocket broadcast from simulation

Maybe have the simulation write current state of creatures to a buffer which is then handed to a seperate component, maybe even one single client of the websocket which has its own hardware to then deal with broadcasting to many clients.

- Simulation tick runs.
- Rapid update of an in memmory state of all creature details is updated in highly optimised way, avoid expensive memmory activity, maybe update in place via pointer? so not new allocations or at least rare allocations needed (preset size of the memmory allocation based on max population?).
This buffer is then broadcast via SSE (Server Sent Events) very efficient? to a seperate component whos job it is to broadcast to many thousands of cleints.

### Questions
How does inbound commands come to simulation, a problem for another day, when we need it?