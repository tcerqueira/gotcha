import { render } from "solid-js/web";
import { GotchaWidget } from "./GotchaWidget";

console.debug("Gotcha script loaded");

// Expose the API globally
(window as any).grecaptcha = {
  ready: (callback: () => void) => {
    // Execute the callback immediately since the DOM is already loaded
    callback();
  },
  render: (container: HTMLElement | string) => {
    const element =
      typeof container === "string"
        ? document.getElementById(container)
        : container;
    if (element) {
      render(() => <GotchaWidget />, element);
    }
  },
};

console.debug("Gotcha API exposed globally");

function init() {
  console.debug("Gotcha initializing...");
  const elements = document.getElementsByClassName("g-recaptcha");
  console.debug(`Found ${elements.length} elements with class 'g-recaptcha'`);
  for (const element of elements) {
    console.debug("Rendering widget in element:", element);
    (window as any).grecaptcha.render(element);
  }
}

// Call init immediately
init();
