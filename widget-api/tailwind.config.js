/** @type {import('tailwindcss').Config} */
module.exports = {
  content: ["./src/**/*.{js,ts,jsx,tsx}"],
  theme: {
    extend: {
      keyframes: {
        "pulse-gradient": {
          "0%, 100%": {
            backgroundSize: "100% 200%",
            backgroundPosition: "0 100%",
          },
          "50%": { backgroundSize: "100% 200%", backgroundPosition: "0 0" },
        },
      },
      animation: {
        "pulse-gradient": "pulse-gradient 1.5s ease-in-out infinite",
      },
    },
  },
  plugins: [],
};
