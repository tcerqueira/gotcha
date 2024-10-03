import { getJsParams } from "./js-params";
import { defaultRenderParams, RenderParams, Widget } from "./lib";
import { Factory } from "./components/yes-no-widget";

type Gotcha = {
  widget: Widget;
  response: string | null;
};

export class GreCaptcha {
  widgets: Gotcha[] = [];

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

    const widget = Factory.create();
    let response: string | null = null;
    const params = {
      ...defaultRenderParams,
      ...parameters,
      callback: (token: string) => {
        response = token;
        parameters.callback && parameters.callback(token);
      },
    };
    let gotcha = {
      widget,
      get response() {
        return response;
      },
      set response(res: string | null) {
        response = res;
      },
    };

    widget.render(element, params);
    return this.widgets.push(gotcha) - 1;
  }

  reset(widgetId?: number) {
    const gotcha = this.getWidget(widgetId);
    if (!gotcha) return;
    gotcha.widget.reset();
    gotcha.response = null;
  }

  getResponse(widgetId?: number): string | null {
    return this.getWidget(widgetId)?.response ?? null;
  }

  private getWidget(widgetId?: number): Gotcha | undefined {
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
