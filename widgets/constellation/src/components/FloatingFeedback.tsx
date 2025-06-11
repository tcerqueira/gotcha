import type { Component } from "solid-js";
import { For } from "solid-js";
import type { FloatingFeedback as FloatingFeedbackType } from "../types";

interface FloatingFeedbackProps {
  feedbacks: FloatingFeedbackType[];
}

const FloatingFeedback: Component<FloatingFeedbackProps> = (props) => {
  return (
    <For each={props.feedbacks}>
      {(feedback) => (
        <div
          class={`absolute pointer-events-none font-bold text-2xl select-none z-20 animate-pulse
            ${feedback.isHit ? "text-green-400" : "text-red-400"}`}
          style={{
            left: `${feedback.x}px`,
            top: `${feedback.y}px`,
            transform: "translate(-50%, -50%)",
            animation: "floatUp 1s ease-out forwards",
          }}
        >
          {feedback.text}
        </div>
      )}
    </For>
  );
};

export default FloatingFeedback;