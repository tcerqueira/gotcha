import { createSignal, onCleanup } from "solid-js";
import { createStore } from "solid-js/store";
import type { Star, FloatingFeedback, Ripple } from "../types";

export const useGameLogic = () => {
  const [gameStarted, setGameStarted] = createSignal(false);
  const [stars, setStars] = createStore<Star[]>([]);
  const [score, setScore] = createSignal(0);
  const [misses, setMisses] = createSignal(0);
  const [gameState, setGameState] = createSignal<'playing' | 'won' | 'lost'>('playing');
  const [targetIdx, setTargetIdx] = createSignal(0);
  const [floatingFeedbacks, setFloatingFeedbacks] = createSignal<FloatingFeedback[]>([]);
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
    if (gameState() !== 'playing') return;
    
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
        x: Math.random() * 80 + 10,
        y: Math.random() * 80 + 10,
      });
    }
    setStars(newStars);
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

    setTimeout(() => {
      setRipples((prev) => prev.filter((r) => r.id !== ripple.id));
    }, 600);
  };

  const startGame = () => {
    setGameStarted(true);
    setScore(0);
    setMisses(0);
    setGameState('playing');
    initializeStars();
  };

  const resetGame = () => {
    setScore(0);
    setMisses(0);
    setGameState('playing');
    randomizeStarPositions();
  };

  const handleStarClick = (star: Star, event: MouseEvent) => {
    if (gameState() !== 'playing') return;

    const rect = (event.currentTarget as HTMLElement).getBoundingClientRect();
    const centerX = rect.left + rect.width / 2;
    const centerY = rect.top + rect.height / 2;

    const isHit = star.id === targetIdx();

    if (isHit) {
      const newScore = score() + 1;
      setScore(newScore);
      setScreenFlash(true);
      setTimeout(() => setScreenFlash(false), 150);
      
      // Check win condition
      if (newScore >= 3) {
        setGameState('won');
        if (timeoutId) {
          clearTimeout(timeoutId);
          timeoutId = undefined;
        }
        return;
      }
    } else {
      const newMisses = misses() + 1;
      setMisses(newMisses);
      
      // Check lose condition
      if (newMisses >= 3) {
        setGameState('lost');
        if (timeoutId) {
          clearTimeout(timeoutId);
          timeoutId = undefined;
        }
        return;
      }
    }

    addFloatingFeedback(centerX, centerY, isHit);
    addRipple(centerX, centerY, isHit);
    randomizeStarPositions();
  };

  onCleanup(() => {
    if (timeoutId) {
      clearTimeout(timeoutId);
    }
  });

  return {
    // State
    gameStarted,
    stars,
    score,
    misses,
    gameState,
    targetIdx,
    floatingFeedbacks,
    ripples,
    screenFlash,
    // Actions
    startGame,
    resetGame,
    handleStarClick,
  };
};