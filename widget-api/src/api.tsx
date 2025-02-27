import "./styles.css";
import { GreCaptcha } from "./grecaptcha";
import { getJsParams } from "./js-params";
import { loadCss } from "./load-css";

loadCss()
  .then(() => {
    // Expose the API globally
    (window as any).grecaptcha = new GreCaptcha();
    const { onload } = getJsParams();
    onload?.();
  })
  .catch((error) => {
    console.error("Error initializing Gotcha:", error);
  });
