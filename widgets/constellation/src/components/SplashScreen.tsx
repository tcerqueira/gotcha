import type { Component } from "solid-js";

interface SplashScreenProps {
  onStart: () => void;
}

const SplashScreen: Component<SplashScreenProps> = (props) => {
  return (
    <div
      class="w-full h-full flex flex-col items-center justify-center cursor-pointer"
      onClick={props.onStart}
    >
      <div class="text-center space-y-8">
        <h1 class="text-6xl md:text-8xl font-bold text-white mb-4">
          Hit the
          <span class="text-yellow-400">⭐</span>
          Stars!
        </h1>
        <p class="text-xl md:text-2xl text-gray-300 mb-8">
          Click the golden star before time runs out
        </p>
        <div class="text-lg md:text-xl text-yellow-400 animate-pulse">
          Click to start
        </div>
      </div>
    </div>
  );
};

export default SplashScreen;