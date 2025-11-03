export interface Position {
  x: number;
  y: number;
}

export interface EntityState {
  id: string;
  position: Position;
  orientation: number;
  radius: number;
}

export interface InterpolatedState extends EntityState {
  previousPosition: Position;
  previousOrientation: number;
  lastUpdateTime: number;
}

export type EntityMap = Map<string, InterpolatedState>;
