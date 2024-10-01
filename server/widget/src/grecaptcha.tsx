import { render } from "solid-js/web";
import { getJsParams } from "./js-params";
import { GotchaWidget } from "./components/GotchaWidget";
import { JSX } from "solid-js/jsx-runtime";

export type RenderParams = {
  sitekey: string;
  theme?: "dark" | "light";
  size?: "compact" | "normal";
  tabindex?: number;
  callback?: (responseToken: string) => void;
  "expired-callback"?: () => void;
  "error-callback"?: () => void;
};

type Widget = {
  element: JSX.Element;
};

export class GreCaptcha {
  widgets: Widget[] = [];

  constructor() {
    const { onload, render, hl } = getJsParams();
    if (render === "explicit") return;

    // the first we find
    let captchaElem = document.getElementsByClassName("g-recaptcha")[0];
    if (captchaElem === undefined) {
      console.error(
        "Could not find 'g-recaptcha' tag. Add 'g-recaptcha' to your class name list.",
      );
      return;
    }

    let params: RenderParams = {
      sitekey: captchaElem.getAttribute("data-sitekey") ?? "",
      theme:
        (captchaElem.getAttribute("data-theme") as
          | "dark"
          | "light"
          | undefined) ?? "light",
      size:
        (captchaElem.getAttribute("data-size") as
          | "compact"
          | "normal"
          | undefined) ?? "normal",
      tabindex: parseInt(captchaElem.getAttribute("data-tabindex") || "0") || 0,
      callback: (window as any)[
        captchaElem.getAttribute("data-callback") || ""
      ],
      "expired-callback": (window as any)[
        captchaElem.getAttribute("data-expired-callback") || ""
      ],
      "error-callback": (window as any)[
        captchaElem.getAttribute("data-error-callback") || ""
      ],
    };

    this.render(captchaElem, params);
  }

  render(container: Element | string, parameters: RenderParams): number | null {
    const element =
      typeof container === "string"
        ? document.getElementById(container)
        : container;

    if (element === null) {
      return null;
    }
    const widget = {
      element: <GotchaWidget />,
    };
    render(() => widget.element, element);
    return this.widgets.push(widget) - 1;
  }

  reset(widgetId?: number) {}

  getResponse(widgetId?: number): string {
    return "not yet implemented";
  }
}
