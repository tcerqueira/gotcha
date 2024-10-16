import { render } from "solid-js/web";
import { RenderParams, WidgetFactory, WidgetMessage } from "@gotcha-widget/lib";
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

    let message = event.data as WidgetMessage;
    switch (message.type) {
      case "response-callback":
        props.callback?.(message.response);
        break;
      case "expired-callback":
        props["expired-callback"]?.();
        break;
      case "error-callback":
        props["error-callback"]?.();
        break;
    }
  };
  onMount(() => {
    window.addEventListener("message", handleMessage);
  });
  onCleanup(() => {
    window.removeEventListener("message", handleMessage);
  });

  // TODO: remove hardcoded URL
  const url = new URL("http://localhost:8080/im-not-a-robot/index.html");
  url.searchParams.append("token", props.sitekey);

  return (
    <div class="gotcha-widget">
      <iframe
        ref={(el) => (iframeElement = el)}
        src={url.href}
        width={304}
        height={78}
        role="presentation"
        sandbox="allow-forms allow-popups allow-same-origin allow-scripts allow-top-navigation allow-modals allow-popups-to-escape-sandbox allow-storage-access-by-user-activation"
      ></iframe>
    </div>
  );
}
