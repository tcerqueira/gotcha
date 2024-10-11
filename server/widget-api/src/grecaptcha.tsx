import { getJsParams } from "./js-params";
import { defaultRenderParams, RenderParams, Widget } from "./lib";
import { Factory } from "./components/gotcha-widget";

type Gotcha = {
  widget: Widget;
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
    const widgetId = this.widgets.length;

    // create wrapper in case the target container already has elements
    // <div style="width: 304px; height: 78px"></div>
    const innerContainer = document.createElement("div");
    innerContainer.style.width = "304px";
    innerContainer.style.height = "78px";
    // this automatically gets sent when inside a form
    // <textarea id="g-response-id" name="g-response-area" style="display: none;">response</textarea>
    const formResponse = document.createElement("textarea");
    formResponse.id =
      widgetId === 0
        ? "g-recaptcha-response"
        : `g-recaptcha-response-${widgetId}`;
    formResponse.name = "g-recaptcha-response";
    formResponse.style.display = "none";
    // add to the DOM
    innerContainer.appendChild(formResponse);
    element.appendChild(innerContainer);

    const widget = Factory.create();
    const params = {
      ...defaultRenderParams,
      ...parameters,
      callback: (token: string) => {
        this.setResponseTextarea(token, widgetId);
        parameters.callback && parameters.callback(token);
      },
    };
    let gotcha = {
      widget,
    };

    widget.render(innerContainer, params);
    return this.widgets.push(gotcha) - 1;
  }

  reset(widgetId?: number) {
    const gotcha = this.getWidget(widgetId);
    if (!gotcha) return;
    gotcha.widget.reset();
    this.setResponseTextarea(null, widgetId);
  }

  getResponse(widgetId?: number): string | null {
    return this.getResponseTextarea(widgetId)?.textContent ?? null;
  }

  private getResponseTextarea(widgetId?: number): Element | null {
    const id = widgetId ?? 0;
    return document.getElementById(
      id === 0 ? "g-recaptcha-response" : `g-recaptcha-response-${id}`,
    );
  }

  private setResponseTextarea(response: string | null, widgetId?: number) {
    let textarea = this.getResponseTextarea(widgetId);
    if (!textarea) return;
    textarea.textContent = response ?? "";
  }

  private getWidget(widgetId?: number): Gotcha | undefined {
    return this.widgets[widgetId ?? 0];
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
