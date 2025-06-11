import type { Component } from "solid-js";

const Animations: Component = () => {
  return (
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
  );
};

export default Animations;