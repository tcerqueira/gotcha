import { Component, Match, Switch } from "solid-js";
import { ChallengeState } from "./types";

type CheckboxWithStatusProps = {
  state: ChallengeState;
  onClick: () => void;
};

export const CheckboxWithStatus: Component<CheckboxWithStatusProps> = (
  props,
) => {
  const isChecked = () => props.state === "verified";
  const isVerifying = () => props.state === "verifying";

  return (
    <div
      class={`w-6 aspect-square border-2 rounded cursor-pointer transition-all duration-200 relative ${
        isChecked() ? "bg-green-500 border-green-500" : "border-gray-300"
      } ${isVerifying() ? "bg-transparent border-transparent" : ""}`}
      onClick={props.onClick}
    >
      <Switch>
        <Match when={isVerifying()}>
          <div class="absolute inset-0">
            <div class="animate-spin w-6 h-6 border-2 border-gray-300 border-t-purple-500 rounded-full" />
          </div>
        </Match>
        <Match when={isChecked()}>
          <svg
            xmlns="http://www.w3.org/2000/svg"
            class="h-5 w-5 text-white"
            viewBox="0 0 20 20"
            fill="currentColor"
          >
            <path
              fill-rule="evenodd"
              d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z"
              clip-rule="evenodd"
            />
          </svg>
        </Match>
      </Switch>
    </div>
  );
};
