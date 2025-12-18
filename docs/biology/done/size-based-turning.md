Implemented biologically realistic turn rate limiting where larger creatures turn more slowly than smaller ones.        
                                                                                                                          
  Changes made:                                                                                                           
                                                                                                                          
  1. physics.rs - Added turn rate allometry constants:                                                                    
    - TURN_RATE_SIZE_EXPONENT = 1.33 (biological scaling)                                                                 
    - MIN_TURN_RATE_DEG = 15.0 (floor for very large creatures)                                                           
    - MAX_TURN_RATE_DEG = 360.0 (cap for tiny creatures)                                                                  
    - TURN_RATE_SPEED_PENALTY = 0.7 (faster = less agile)                                                                 
  2. systems.rs - Updated movement system with size-dependent turn rate:                                                  
    - turn_rate = BASE / size^1.33 (clamped to [15°, 360°])                                                               
    - Speed penalty: speed_factor = 1.0 - 0.7 × (speed/max_speed)²                                                        
    - At max speed, creatures retain 30% of turn ability                                                                  
  3. New tests:                                                                                                           
    - test_large_creature_turns_slower_than_small - Verifies 5m creature turns slower than 0.5m                           
    - test_turn_rate_clamped_to_min_for_very_large - Verifies 100m creature doesn't go below MIN                          
    - test_turn_rate_clamped_to_max_for_very_small - Verifies 0.01m creature doesn't exceed MAX                           
    - test_speed_penalty_reduces_turn_rate - Verifies fast creatures turn slower than slow ones                           
                                                                                                                          
  Emergent behaviors:                                                                                                     
  | Size | Base Turn Rate | At Max Speed |                                                                                
  |------|----------------|--------------|                                                                                
  | 0.5m | ~290°/s        | ~87°/s       |                                                                                
  | 1.0m | 180°/s         | 54°/s        |                                                                                
  | 2.0m | ~72°/s         | ~22°/s       |                                                                                
  | 5.0m | ~21°/s         | ~6°/s        |                                                                                
                                                                                                                          
  Small creatures now excel at quick pivots and evasion, while large creatures must commit to paths with wide turning     
  arcs.                                            