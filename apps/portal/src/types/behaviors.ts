export const BehaviorDiscriminant = {
  Catatonic: 0,
  Seeking: 1,
  Wandering: 2,
} as const;

export const BehaviorNames: Record<number, string> = {
  0: 'Catatonic',
  1: 'Seeking',
  2: 'Wandering',
};

export type BehaviorMode = keyof typeof BehaviorDiscriminant;

export function getBehaviorName(discriminant: number): string {
  return BehaviorNames[discriminant] ?? 'Unknown';
}
