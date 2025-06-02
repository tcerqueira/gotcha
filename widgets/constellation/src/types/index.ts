export interface Star {
  id: number;
  x: number;
  y: number;
}

export interface FloatingFeedback {
  id: number;
  x: number;
  y: number;
  text: string;
  isHit: boolean;
}

export interface Ripple {
  id: number;
  x: number;
  y: number;
  isHit: boolean;
}