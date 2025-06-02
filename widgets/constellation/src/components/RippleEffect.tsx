import type { Component } from "solid-js";
import { For } from "solid-js";
import type { Ripple } from "../types";

interface RippleEffectProps {
  ripples: Ripple[];
}

const RippleEffect: Component<RippleEffectProps> = (props) => {
  return (
    <For each={props.ripples}>
      {(ripple) => (
        <div
          class="absolute pointer-events-none z-10"
          style={{
            left: `${ripple.x}px`,
            top: `${ripple.y}px`,
            transform: "translate(-50%, -50%)",
          }}
        >
          <div
            class={`w-4 h-4 rounded-full border-2 opacity-75
              ${ripple.isHit ? "border-yellow-400" : "border-red-400"}`}
            style={{
              animation: "ripple 0.6s ease-out forwards",
            }}
          />
        </div>
      )}
    </For>
  );
};

export default RippleEffect;