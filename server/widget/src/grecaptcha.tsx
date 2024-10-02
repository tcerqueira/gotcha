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
  container: Element;
  renderParams: RenderParams;
  response: string | null;
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

    let response: string | null = null;
    const widget = {
      element: (
        <GotchaWidget
          onSuccess={(res: string) => {
            response = res;
            alert(response);
          }}
          onFailure={() => {
            response = null;
            alert("beep boop");
          }}
        />
      ),
      renderParams: parameters,
      container: element,
      get response() {
        return response;
      },
    };
    render(() => widget.element, element);
    return this.widgets.push(widget) - 1;
  }

  reset(widgetId?: number) {
    const widget = this.getWidget(widgetId);
    if (!widget) return;
    while (widget.container.firstChild) {
      widget.container.removeChild(widget.container.lastChild!);
    }
    this.render(widget.container, widget?.renderParams);
  }

  getResponse(widgetId?: number): string | null {
    return this.getWidget(widgetId)?.response ?? null;
  }

  private getWidget(widgetId?: number): Widget | undefined {
    return this.widgets[widgetId ?? 0];
  }

  private getParamsFromContainer(container: Element): RenderParams {
    return {
      sitekey: container.getAttribute("data-sitekey") ?? "",
      theme:
        (container.getAttribute("data-theme") as
          | "dark"
          | "light"
          | undefined) ?? "light",
      size:
        (container.getAttribute("data-size") as
          | "compact"
          | "normal"
          | undefined) ?? "normal",
      tabindex: parseInt(container.getAttribute("data-tabindex") || "0") || 0,
      callback: (window as any)[container.getAttribute("data-callback") ?? ""],
      "expired-callback": (window as any)[
        container.getAttribute("data-expired-callback") ?? ""
      ],
      "error-callback": (window as any)[
        container.getAttribute("data-error-callback") ?? ""
      ],
    };
  }
}
