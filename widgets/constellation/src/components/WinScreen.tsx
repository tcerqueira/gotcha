import type { Component } from "solid-js";

interface WinScreenProps {
  score: number;
  onRestart: () => void;
}

const WinScreen: Component<WinScreenProps> = (props) => {
  return (
    <div
      class="w-full h-full flex flex-col items-center justify-center cursor-pointer"
      onClick={props.onRestart}
    >
      <div class="text-center space-y-8">
        <h1 class="text-6xl md:text-8xl font-bold text-yellow-400 mb-4">
          You Won!
        </h1>
        <p class="text-2xl md:text-3xl text-green-400 mb-4">
          Amazing! You hit {props.score} golden stars!
        </p>
        <p class="text-xl md:text-2xl text-gray-300 mb-8">
          Your reflexes are stellar! ⭐
        </p>
        <div class="text-lg md:text-xl text-yellow-400 animate-pulse">
          Click to play again
        </div>
      </div>
    </div>
  );
};

export default WinScreen;
