import { GreCaptcha } from "./grecaptcha";
import { getJsParams } from "./js-params";

// Expose the API globally
(window as any).grecaptcha = new GreCaptcha();
const { onload } = getJsParams();
onload && onload();
