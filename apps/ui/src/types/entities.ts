export interface Vec3 {
  x: number;
  y: number;
  z: number;
}

export interface EntityState {
  id: string;
  prevPosition: Vec3;
  currentPosition: Vec3;
  prevTimestamp: number;
  currentTimestamp: number;
  tick: number;
}

export enum ConnectionState {
  Connecting = 'Connecting',
  Connected = 'Connected',
  Disconnected = 'Disconnected',
  Reconnecting = 'Reconnecting',
}
