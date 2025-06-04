import { Component } from "solid-js";

export type CheckboxState = "blank" | "checking" | "checked";

interface CheckboxProps {
  state: CheckboxState;
  onClick: () => void;
  disabled?: boolean;
  className?: string;
}

const Checkbox: Component<CheckboxProps> = (props) => {
  const renderState = () => {
    switch (props.state) {
      case "blank":
        return (
          <div class="size-6 border-2 border-gray-300 rounded bg-white hover:border-gray-400 transition-colors duration-200" />
        );
      case "checking":
        return (
          <svg
            class="animate-spin size-6 text-sky-500"
            xmlns="http://www.w3.org/2000/svg"
            fill="none"
            viewBox="0 0 24 24"
          >
            <circle
              class="opacity-25"
              cx="12"
              cy="12"
              r="10"
              stroke="currentColor"
              stroke-width="4"
            />
            <path
              class="opacity-75"
              fill="currentColor"
              d="m4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
            />
          </svg>
        );
      case "checked":
        return (
          <svg
            class="size-6 text-green-600"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
            stroke-width="3"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              d="M5 13l4 4L19 7"
            />
          </svg>
        );
    }
  };

  return (
    <button
      type="button"
      onClick={props.onClick}
      disabled={props.disabled}
      class={`
          inline-flex
          items-center
          justify-center
          transition-opacity
          duration-200
          ${props.disabled ? "opacity-50 cursor-not-allowed" : "cursor-pointer"}
          ${props.className || ""}
        `
        .trim()
        .replace(/\s+/g, " ")}
    >
      {renderState()}
    </button>
  );
};

export default Checkbox;
