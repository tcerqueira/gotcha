import { render } from "solid-js/web";
import { RenderParams, WidgetFactory } from "@gotcha-widget/lib";
import { onCleanup, onMount } from "solid-js";

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
        containerElem.getElementsByClassName("gotcha-widget")[0]?.remove();
        renderWidget(containerElem, params!);
      },
    };
  },
};

export type GotchaWidgetProps = RenderParams;

export function GotchaWidget(props: GotchaWidgetProps) {
  let iframeElement: HTMLIFrameElement | null = null;

  const handleMessage = (event: MessageEvent<any>) => {
    if (
      // Always check the origin of the message
      event.origin !== "http://localhost:8080" ||
      // Only listen for events coming from this iframe and no other
      event.source !== iframeElement?.contentWindow
    )
      return;

    props.callback?.(event.data);
  };
  onMount(() => {
    window.addEventListener("message", handleMessage);
  });
  onCleanup(() => {
    window.removeEventListener("message", handleMessage);
  });

  // TODO: hardcoded URL
  return (
    <div class="gotcha-widget">
      <iframe
        ref={(el) => (iframeElement = el)}
        src="http://localhost:8080/im-not-a-robot/index.html"
        width={304}
        height={78}
        role="presentation"
        sandbox="allow-forms allow-popups allow-same-origin allow-scripts allow-top-navigation allow-modals allow-popups-to-escape-sandbox allow-storage-access-by-user-activation"
      ></iframe>
    </div>
  );
}
