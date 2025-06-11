import type { Component } from "solid-js";

interface BackgroundLogoProps {
  src: string;
  alt?: string;
}

const BackgroundLogo: Component<BackgroundLogoProps> = (props) => {
  return (
    <div class="absolute inset-0 flex items-center justify-center pointer-events-none">
      <img
        src={props.src}
        alt={props.alt || "Logo"}
        class="w-full h-full opacity-10 select-none object-contain"
      />
    </div>
  );
};

export default BackgroundLogo;
