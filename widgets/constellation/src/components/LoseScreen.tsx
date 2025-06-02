import type { Component } from "solid-js";

interface LoseScreenProps {
  misses: number;
  onRestart: () => void;
}

const LoseScreen: Component<LoseScreenProps> = (props) => {
  return (
    <div
      class="w-full h-full flex flex-col items-center justify-center cursor-pointer"
      onClick={props.onRestart}
    >
      <div class="text-center space-y-8">
        <h1 class="text-6xl md:text-8xl font-bold text-red-400 mb-4">
          Game Over!
        </h1>
        <p class="text-2xl md:text-3xl text-red-300 mb-4">
          You missed {props.misses} stars in a row!
        </p>
        <p class="text-xl md:text-2xl text-gray-300 mb-8">
          Don't give up - try again! 🌟
        </p>
        <div class="text-lg md:text-xl text-yellow-400 animate-pulse">
          Click to try again
        </div>
      </div>
    </div>
  );
};

export default LoseScreen;
