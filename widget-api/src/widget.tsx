import { render } from "solid-js/web";
import { createSignal } from "solid-js";
import { RenderParams } from "./grecaptcha";
import { GotchaWidget } from "./components/gotcha-widget";
import { GotchaWidgetProps } from "./components/types";

export interface Widget {
  render: (container: Element, parameters: RenderParams) => void;
  reset: () => void;
}

export type LiveState = "live" | "expired";

export function createWidget(): Widget {
  let containerElem: Element | undefined;
  let params: GotchaWidgetProps | undefined;
  const [state, setState] = createSignal<LiveState>("live");
  let timeout: NodeJS.Timeout | undefined;

  const renderWidget = (container: Element, parameters: RenderParams) => {
    containerElem = container;
    params = {
      ...parameters,
      "expired-callback": () => {
        setState("expired");
        parameters["expired-callback"]?.();
      },
      callback: (token) => {
        setState("live");
        parameters.callback?.(token);
        clearTimeout(timeout);
        timeout = setTimeout(() => params?.["expired-callback"]?.(), 30000);
      },
      liveState: state,
    };

    render(
      () => <GotchaWidget {...(params as GotchaWidgetProps)} />,
      containerElem,
    );
  };

  return {
    render: renderWidget,
    reset: () => {
      setState("live");
      clearTimeout(timeout);

      if (!containerElem) return;
      containerElem.getElementsByClassName("gotcha-widget")[0]?.remove();
      renderWidget(containerElem, params!);
    },
  };
}
