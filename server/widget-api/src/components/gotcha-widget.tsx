import { render } from "solid-js/web";
import { RenderParams, WidgetMessage } from "@gotcha-widget/lib";
import { createResource, Match, onCleanup, onMount, Switch } from "solid-js";

export interface Widget {
  render: (container: Element, parameters: RenderParams) => void;
  reset: () => void;
}

export function createWidget(): Widget {
  let containerElem: Element | undefined;
  let params: RenderParams | undefined;

  const renderWidget = (container: Element, parameters: RenderParams) => {
    containerElem = container;
    params = parameters;

    render(() => <GotchaWidget {...(params as RenderParams)} />, containerElem);
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
}

export type GotchaWidgetProps = RenderParams;

export function GotchaWidget(props: GotchaWidgetProps) {
  let iframeElement: HTMLIFrameElement | null = null;

  const handleMessage = (event: MessageEvent<WidgetMessage>) => {
    if (
      // Always check the origin of the message
      // event.origin !== "http://localhost:8080" ||
      // Only listen for events coming from this iframe and no other
      event.source !== iframeElement?.contentWindow
    )
      return;

    let message = event.data;
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
  const [challenge] = createResource(props.sitekey, fetchChallenge);

  return (
    <div class="gotcha-widget">
      <Switch>
        <Match when={challenge.loading}>
          <p>Loading...</p>
        </Match>
        <Match when={challenge.error}>
          <span>Error: {challenge.error}</span>
        </Match>
        <Match when={challenge()}>
          <div
            class="border-2 border-purple-200 rounded box-content bg-gray-50"
            style={{ width: `${challenge()?.width ?? 304}px` }}
          >
            <iframe
              class="border-none overflow-hidden m-0 p-0 focus-visible:outline-none"
              ref={(el) => (iframeElement = el)}
              src={challenge()?.url}
              width={challenge()?.width}
              height={challenge()?.height}
              role="presentation"
              sandbox="allow-forms allow-popups allow-same-origin allow-scripts allow-top-navigation allow-modals allow-popups-to-escape-sandbox allow-storage-access-by-user-activation"
            ></iframe>
            <div class="bg-gray-200">
              <p class="text-right m-0">Gotcha</p>
            </div>
          </div>
        </Match>
      </Switch>
    </div>
  );
}

type Challenge = {
  url: string;
  width: number;
  height: number;
};

async function fetchChallenge(token: string): Promise<Challenge> {
  const origin = new URL(import.meta.url).origin;
  const url = new URL(`${origin}/api/challenge`);
  url.searchParams.append("token", token);

  const response = await fetch(url);
  return (await response.json()) as Challenge;
}
