import { render } from "solid-js/web";
import { RenderParams, Widget, WidgetFactory } from "../lib";

export const Factory: WidgetFactory = {
  create: function (): Widget {
    // TODO: remove this hack, control state directly
    let containerElem: Element | undefined;
    let params: RenderParams | undefined;

    const renderWidget = (container: Element, parameters: RenderParams) => {
      containerElem = container;
      params = parameters;

      render(
        () => (
          <GotchaWidget
            onSuccess={parameters.callback}
            onFailure={parameters["error-callback"]}
          />
        ),
        containerElem,
      );
    };

    return {
      render: renderWidget,
      reset: () => {
        // TODO: control state instead of clearing and rerendering
        if (!containerElem) return;
        containerElem.getElementsByClassName("yes-no-widget")[0]?.remove();
        renderWidget(containerElem, params!);
      },
    };
  },
};

export type GotchaWidgetProps = {
  onSuccess?: (response: string) => void;
  onFailure?: () => void;
};

export function GotchaWidget(props: GotchaWidgetProps) {
  return (
    <div class="yes-no-widget">
      <span>Are you a robot?</span>
      <button type="button" onClick={props.onFailure}>
        YES
      </button>
      <button
        type="button"
        onClick={() => props.onSuccess && props.onSuccess("congratz")}
      >
        NO
      </button>
    </div>
  );
}
