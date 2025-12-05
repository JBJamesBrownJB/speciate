10. Adding ONLY catatonic crits increases movement system latency... make no sense!

11. Use topological neighbour sorting for crits near to the camera but fall back to pseudo random ones for ones away from camera?
This means we have to have the frontend send the current viewport dimensions and the backend has to dynamically apply the LOD (in this case topological vs psuedo random neighbour sorting) approach. This likely introduces the key concept of LOD AI into our system and probably a broader sprint. SO instead, just start detailing next sprint, where we will implement LOD AI structure and test it with this as first example.

12. improve spatial grid rebuilt latency SEE all the parallel-*.md files 