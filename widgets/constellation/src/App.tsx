import type { Component } from "solid-js";
import { createSignal, onMount, For, onCleanup, createEffect } from "solid-js";
import { createStore } from "solid-js/store";
import starknetLogo from "./assets/starknet-logo.svg";

interface Star {
  id: number;
  x: number;
  y: number;
}

interface FloatingFeedback {
  id: number;
  x: number;
  y: number;
  text: string;
  isHit: boolean;
}

interface Ripple {
  id: number;
  x: number;
  y: number;
  isHit: boolean;
}

const App: Component = () => {
  const [gameStarted, setGameStarted] = createSignal(false);
  const [stars, setStars] = createStore<Star[]>([]);
  const [score, setScore] = createSignal(0);
  const [targetIdx, setTargetIdx] = createSignal(0);
  const [floatingFeedbacks, setFloatingFeedbacks] = createSignal<
    FloatingFeedback[]
  >([]);
  const [ripples, setRipples] = createSignal<Ripple[]>([]);
  const [screenFlash, setScreenFlash] = createSignal(false);

  let timeoutId: number | undefined;
  let feedbackIdCounter = 0;

  const startTimeout = () => {
    if (timeoutId) {
      clearTimeout(timeoutId);
    }
    timeoutId = setTimeout(() => {
      randomizeStarPositions();
    }, 8000);
  };

  const randomizeStarPositions = () => {
    setTargetIdx(Math.floor(Math.random() * 5));
    for (let i = 0; i < stars.length; i++) {
      setStars(i, {
        x: Math.random() * 80 + 10,
        y: Math.random() * 80 + 10,
      });
    }
    startTimeout();
  };

  const initializeStars = () => {
    const newStars: Star[] = [];
    for (let i = 0; i < 5; i++) {
      newStars.push({
        id: i,
        x: Math.random() * 80 + 10, // 10-90% of screen width
        y: Math.random() * 80 + 10, // 10-90% of screen height
      });
    }
    setStars(newStars);
    // Reuse randomization to ensure positions are set properly
    randomizeStarPositions();
  };

  const addFloatingFeedback = (x: number, y: number, isHit: boolean) => {
    const feedback: FloatingFeedback = {
      id: feedbackIdCounter++,
      x,
      y,
      text: isHit ? "+1" : "Miss!",
      isHit,
    };

    setFloatingFeedbacks((prev) => [...prev, feedback]);

    // Remove after animation
    setTimeout(() => {
      setFloatingFeedbacks((prev) => prev.filter((f) => f.id !== feedback.id));
    }, 1000);
  };

  const addRipple = (x: number, y: number, isHit: boolean) => {
    const ripple: Ripple = {
      id: feedbackIdCounter++,
      x,
      y,
      isHit,
    };

    setRipples((prev) => [...prev, ripple]);

    // Remove after animation
    setTimeout(() => {
      setRipples((prev) => prev.filter((r) => r.id !== ripple.id));
    }, 600);
  };

  const startGame = () => {
    setGameStarted(true);
    initializeStars();
  };

  const handleStarClick = (star: Star, event: MouseEvent) => {
    const rect = (event.currentTarget as HTMLElement).getBoundingClientRect();
    const centerX = rect.left + rect.width / 2;
    const centerY = rect.top + rect.height / 2;

    const isHit = star.id === targetIdx();

    if (isHit) {
      setScore(score() + 1);
      // Screen flash for hits
      setScreenFlash(true);
      setTimeout(() => setScreenFlash(false), 150);
    }

    // Add visual feedback
    addFloatingFeedback(centerX, centerY, isHit);
    addRipple(centerX, centerY, isHit);

    // Generate new positions regardless of hit or miss
    randomizeStarPositions();
  };

  onMount(() => {
    // Game will start when splash screen is clicked
  });

  onCleanup(() => {
    if (timeoutId) {
      clearTimeout(timeoutId);
    }
  });

  return (
    <div class="w-screen h-screen relative bg-slate-900 overflow-hidden">
      {/* Background logo */}
      <div class="absolute inset-0 flex items-center justify-center pointer-events-none">
        <img
          src={starknetLogo}
          alt="Starknet"
          class="w-full h-full opacity-10 select-none object-contain"
        />
      </div>

      {!gameStarted() ? (
        // Splash Screen
        <div
          class="w-full h-full flex flex-col items-center justify-center cursor-pointer"
          onClick={startGame}
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
      ) : (
        // Game Screen
        <>
          {/* Screen flash overlay */}
          <div
            class={`absolute inset-0 pointer-events-none transition-opacity duration-150 ${
              screenFlash() ? "opacity-20 bg-yellow-400" : "opacity-0"
            }`}
          />

          {/* Score display */}
          <div class="absolute top-4 left-4 text-white text-2xl font-bold z-10">
            Score: {score()}
          </div>
          <For each={stars}>
            {(star) => (
              <div
                class="absolute cursor-pointer select-none w-8 h-8 sm:w-10 sm:h-10 md:w-12 md:h-12 transition-all duration-300 ease-in-out"
                style={{ left: `${star.x}%`, top: `${star.y}%` }}
                onClick={(e) => handleStarClick(star, e)}
              >
                <svg
                  viewBox="0 0 24 24"
                  fill="currentColor"
                  class={`w-full h-full transition-colors duration-300 ease-in-out ${star.id === targetIdx() ? "text-yellow-400" : "text-white"}`}
                >
                  <path d="M12 2l3.09 6.26L22 9.27l-5 4.87 1.18 6.88L12 17.77l-6.18 3.25L7 14.14 2 9.27l6.91-1.01L12 2z" />
                </svg>
              </div>
            )}
          </For>

          {/* Floating feedback texts */}
          <For each={floatingFeedbacks()}>
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

          {/* Ripple effects */}
          <For each={ripples()}>
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
        </>
      )}

      <style>{`
        @keyframes floatUp {
          0% {
            opacity: 1;
            transform: translate(-50%, -50%) scale(1);
          }
          100% {
            opacity: 0;
            transform: translate(-50%, -150%) scale(1.2);
          }
        }

        @keyframes ripple {
          0% {
            transform: scale(1);
            opacity: 0.75;
          }
          100% {
            transform: scale(8);
            opacity: 0;
          }
        }
      `}</style>
    </div>
  );
};

export default App;
