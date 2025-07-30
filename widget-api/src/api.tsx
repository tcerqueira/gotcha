import { GotchaCaptcha } from "./gotcha-captcha";
import { getJsParams } from "./js-params";
import { loadCss } from "./load-css";
import "./styles.css";

loadCss()
  .then(() => {
    // Expose the API globally
    (window as any).gotcha = new GotchaCaptcha();
    const { onload } = getJsParams();
    onload?.();
  })
  .catch((error) => {
    console.error("Error initializing Gotcha:", error);
  });
