import type { Component } from "solid-js";
import { useGameLogic } from "./hooks/useGameLogic";
import SplashScreen from "./components/SplashScreen";
import GameScreen from "./components/GameScreen";
import WinScreen from "./components/WinScreen";
import LoseScreen from "./components/LoseScreen";
import BackgroundLogo from "./components/BackgroundLogo";
import Animations from "./components/Animations";
import starknetLogo from "./assets/starknet-logo.svg";

const App: Component = () => {
  const {
    gameStarted,
    stars,
    score,
    misses,
    gameState,
    targetIdx,
    floatingFeedbacks,
    ripples,
    screenFlash,
    startGame,
    resetGame,
    handleStarClick,
  } = useGameLogic();

  return (
    <div class="w-screen h-screen relative bg-slate-900 overflow-hidden">
      <BackgroundLogo src={starknetLogo} alt="Starknet" />
      
      {!gameStarted() ? (
        <SplashScreen onStart={startGame} />
      ) : gameState() === 'won' ? (
        <WinScreen score={score()} onRestart={resetGame} />
      ) : gameState() === 'lost' ? (
        <LoseScreen misses={misses()} onRestart={resetGame} />
      ) : (
        <GameScreen
          stars={stars}
          score={score()}
          misses={misses()}
          targetIdx={targetIdx()}
          floatingFeedbacks={floatingFeedbacks()}
          ripples={ripples()}
          screenFlash={screenFlash()}
          onStarClick={handleStarClick}
        />
      )}
      
      <Animations />
    </div>
  );
};

export default App;