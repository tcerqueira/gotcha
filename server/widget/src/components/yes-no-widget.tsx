import { render } from "solid-js/web";
import { generateResponseToken, RenderParams, WidgetFactory } from "../lib";

export const Factory: WidgetFactory = {
  create: () => {
    let containerElem: Element | undefined;
    let params: RenderParams | undefined;

    const renderWidget = (container: Element, parameters: RenderParams) => {
      containerElem = container;
      params = parameters;

      render(
        () => <GotchaWidget {...(params as RenderParams)} />,
        containerElem,
      );
    };

    return {
      render: renderWidget,
      reset: () => {
        // TODO: remove this hack, control state directly
        if (!containerElem) return;
        containerElem.getElementsByClassName("yes-no-widget")[0]?.remove();
        renderWidget(containerElem, params!);
      },
    };
  },
};

export type GotchaWidgetProps = RenderParams;

export function GotchaWidget(props: GotchaWidgetProps) {
  return (
    <div class="yes-no-widget">
      <span>I'm not a robot</span>
      <button
        type="button"
        onClick={() =>
          props.callback &&
          props.callback(generateResponseToken(true, props.sitekey))
        }
      >
        PASS
      </button>
      <button
        type="button"
        onClick={() => {
          props.callback &&
            props.callback(generateResponseToken(false, props.sitekey));
        }}
      >
        FAIL
      </button>
    </div>
  );
}
