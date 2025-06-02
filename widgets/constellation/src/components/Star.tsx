import type { Component } from "solid-js";
import type { Star as StarType } from "../types";

interface StarProps {
  star: StarType;
  targetIdx: number;
  onClick: (star: StarType, event: MouseEvent) => void;
}

const Star: Component<StarProps> = (props) => {
  return (
    <div
      class="absolute cursor-pointer select-none w-8 h-8 sm:w-10 sm:h-10 md:w-12 md:h-12 transition-all duration-300 ease-in-out"
      style={{ left: `${props.star.x}%`, top: `${props.star.y}%` }}
      onClick={(e) => props.onClick(props.star, e)}
    >
      <svg
        viewBox="0 0 24 24"
        fill="currentColor"
        class={`w-full h-full transition-colors duration-300 ease-in-out ${
          props.star.id === props.targetIdx ? "text-yellow-400" : "text-white"
        }`}
      >
        <path d="M12 2l3.09 6.26L22 9.27l-5 4.87 1.18 6.88L12 17.77l-6.18 3.25L7 14.14 2 9.27l6.91-1.01L12 2z" />
      </svg>
    </div>
  );
};

export default Star;