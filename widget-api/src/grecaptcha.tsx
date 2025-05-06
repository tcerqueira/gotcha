import { getJsParams } from "./js-params";
import { render } from "solid-js/web";
import { createWidget, Widget } from "./widget";

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

    const params = this.getParamsFromContainer(captchaElem);
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

    const widgetId = this.widgets.length;
    const containerId =
      widgetId === 0
        ? "g-recaptcha-container"
        : `g-recaptcha-container-${widgetId}`;
    const textareaId =
      widgetId === 0
        ? "g-recaptcha-response"
        : `g-recaptcha-response-${widgetId}`;

    const innerContainer = (
      <div id={containerId}>
        <textarea
          id={textareaId}
          name="g-recaptcha-response"
          style="display: none;"
        ></textarea>
      </div>
    );

    const widget = createWidget();
    const params = {
      ...defaultRenderParams,
      ...parameters,
      callback: (token: string) => {
        this.setResponseTextarea(token, widgetId);
        parameters.callback?.(token);
      },
      "expired-callback": () => {
        this.setResponseTextarea(null, widgetId);
        parameters["expired-callback"]?.();
      },
      "error-callback": () => {
        this.setResponseTextarea(null, widgetId);
        parameters["error-callback"]?.();
      },
    };

    render(() => innerContainer, element);
    widget.render(document.getElementById(containerId)!, params);
    return this.widgets.push(widget) - 1;
  }

  reset(widgetId?: number) {
    const gotcha = this.getWidget(widgetId);
    if (!gotcha) return;
    gotcha.reset();
    this.setResponseTextarea(null, widgetId);
  }

  getResponse(widgetId?: number): string | null {
    return this.getResponseElement(widgetId)?.textContent ?? null;
  }

  private getResponseElement(widgetId: number = 0): Element | null {
    return document.getElementById(
      widgetId === 0
        ? "g-recaptcha-response"
        : `g-recaptcha-response-${widgetId}`,
    );
  }

  private setResponseTextarea(response: string | null, widgetId?: number) {
    let textarea = this.getResponseElement(widgetId);
    if (!textarea) return;
    textarea.textContent = response ?? "";
  }

  private getWidget(widgetId: number = 0): Widget | undefined {
    return this.widgets[widgetId];
  }

  private getParamsFromContainer(container: Element): RenderParams {
    return {
      sitekey: container.getAttribute("data-sitekey") ?? "",
      theme: container.getAttribute("data-theme") as
        | "dark"
        | "light"
        | undefined,
      size: container.getAttribute("data-size") as
        | "compact"
        | "normal"
        | undefined,
      tabindex: parseInt(container.getAttribute("data-tabindex") || "0") || 0,
      callback:
        (window as any)[container.getAttribute("data-callback") ?? ""] ?? null,
      "expired-callback":
        (window as any)[
          container.getAttribute("data-expired-callback") ?? ""
        ] ?? null,
      "error-callback":
        (window as any)[container.getAttribute("data-error-callback") ?? ""] ??
        null,
    };
  }
}

/**
 * Configuration parameters for rendering the captcha widget
 */
export type RenderParams = {
  /** API key for site verification */
  sitekey: string;
  /** Widget theme appearance */
  theme?: "dark" | "light";
  /** Widget size configuration */
  size?: "compact" | "normal" | "invisible";
  /** Badge position in the widget */
  badge?: "bottomright" | "bottomleft" | "inline";
  /** Tab index for accessibility */
  tabindex?: number;
  /** Callback function on successful verification */
  callback?: (token: string) => void;
  /** Callback function when challenge expires */
  "expired-callback"?: () => void;
  /** Callback function on error */
  "error-callback"?: () => void;
  /** For plugin owners to not interfere with existing reCAPTCHA installations on a page.
   * If true, this reCAPTCHA instance will be part of a separate ID space
   */
  isolated?: boolean;
};

/**
 * Default configuration parameters for the widget
 */
export const defaultRenderParams: RenderParams = {
  sitekey: "",
  theme: undefined,
  size: "normal",
  badge: "bottomright",
  tabindex: 0,
  isolated: false,
};
