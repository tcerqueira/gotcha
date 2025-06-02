import type { Component } from "solid-js";
import { For } from "solid-js";
import type { Star as StarType, FloatingFeedback as FloatingFeedbackType, Ripple } from "../types";
import Star from "./Star";
import FloatingFeedback from "./FloatingFeedback";
import RippleEffect from "./RippleEffect";

interface GameScreenProps {
  stars: StarType[];
  score: number;
  misses: number;
  targetIdx: number;
  floatingFeedbacks: FloatingFeedbackType[];
  ripples: Ripple[];
  screenFlash: boolean;
  onStarClick: (star: StarType, event: MouseEvent) => void;
}

const GameScreen: Component<GameScreenProps> = (props) => {
  return (
    <>
      {/* Screen flash overlay */}
      <div
        class={`absolute inset-0 pointer-events-none transition-opacity duration-150 ${
          props.screenFlash ? "opacity-20 bg-yellow-400" : "opacity-0"
        }`}
      />

      {/* Score display */}
      <div class="absolute top-4 left-4 text-white text-2xl font-bold z-10">
        Score: {props.score}
      </div>
      
      {/* Miss counter */}
      <div class="absolute top-4 right-4 text-white text-2xl font-bold z-10">
        Misses: {props.misses}/3
      </div>

      {/* Stars */}
      <For each={props.stars}>
        {(star) => (
          <Star
            star={star}
            targetIdx={props.targetIdx}
            onClick={props.onStarClick}
          />
        )}
      </For>

      {/* Floating feedback texts */}
      <FloatingFeedback feedbacks={props.floatingFeedbacks} />

      {/* Ripple effects */}
      <RippleEffect ripples={props.ripples} />
    </>
  );
};

export default GameScreen;