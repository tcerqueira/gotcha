import { Component } from "solid-js";
import CheckingSvg from "./icons/checking";
import CheckedSvg from "./icons/checked";

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
        return <CheckingSvg />;
      case "checked":
        return <CheckedSvg />;
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
