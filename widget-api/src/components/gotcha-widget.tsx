import { render } from "solid-js/web";
import { SearchParams, WidgetMessage, Interaction } from "@gotcha-widget/lib";
import {
  Accessor,
  createResource,
  createSignal,
  Match,
  onCleanup,
  onMount,
  Show,
  Switch,
} from "solid-js";
import { defaultRenderParams, RenderParams } from "../grecaptcha";
import ImNotRobot, { AnalysisResponse } from "./im-not-a-robot";
import Modal from "./modal";

export interface Widget {
  render: (container: Element, parameters: RenderParams) => void;
  reset: () => void;
}

type State = "live" | "expired";

export function createWidget(): Widget {
  let containerElem: Element | undefined;
  let params: GotchaWidgetProps | undefined;
  const [state, setState] = createSignal<State>("live");
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
      state,
    };

    render(
      () => <GotchaWidget {...(params as GotchaWidgetProps)} />,
      containerElem,
    );
  };

  return {
    render: renderWidget,
    reset: () => {
      if (!containerElem) return;
      containerElem.getElementsByClassName("gotcha-widget")[0]?.remove();
      renderWidget(containerElem, params!);
    },
  };
}

type AdditionalParams = {
  state: Accessor<State>;
};
export type GotchaWidgetProps = RenderParams & AdditionalParams;

export function GotchaWidget(props: GotchaWidgetProps) {
  const [showChallenge, setShowChallenge] = createSignal(false);
  let iframeElement: HTMLIFrameElement | null = null;
  const [challenge] = createResource(props.sitekey, fetchChallenge);

  const handlePreChallengeResponse = (response: AnalysisResponse) => {
    if (response.result === "success") {
      props.callback?.(response.response.token);
    } else {
      setShowChallenge(true);
    }
  };

  const handleMessage = async (event: MessageEvent<WidgetMessage>) => {
    const challengeData = challenge();
    if (!challengeData) return;

    if (
      // Always check the origin of the message
      event.origin !== new URL(challengeData.url).origin ||
      // Only listen for events coming from this iframe and no other
      event.source !== iframeElement?.contentWindow
    )
      return;

    let message = event.data;
    switch (message.type) {
      case "response-callback":
        let response = await processChallenge(
          props.sitekey,
          message.success,
          challengeData.url,
          message.interactions,
        );
        if (response !== null) {
          props.callback?.(response);
        } else {
          props["error-callback"]?.();
        }
        break;
      case "error-callback":
        props["error-callback"]?.();
        break;
    }
    setShowChallenge(false);
  };
  onMount(() => {
    window.addEventListener("message", handleMessage);
  });
  onCleanup(() => {
    window.removeEventListener("message", handleMessage);
  });

  const params: SearchParams = {
    k: props.sitekey,
    theme: props.theme,
    size: props.size,
    badge: props.badge,
    sv: window.location.origin,
  };

  return (
    <div class="gotcha-widget inline-block">
      <div class={`border-2 border-purple-200 rounded box-content bg-gray-50`}>
        <ImNotRobot params={props} onResponse={handlePreChallengeResponse} />
        <Show when={showChallenge()}>
          <Modal open={showChallenge()}>
            <Switch>
              <Match when={challenge.loading}>
                <p>Loading...</p>
              </Match>
              <Match when={challenge.error}>
                <span>Error {challenge.error}</span>
              </Match>
              <Match when={challenge()}>
                <div class={`w-[${challenge()!.width}px]`}>
                  <iframe
                    class="border-none overflow-hidden m-0 p-0 focus-visible:outline-none"
                    ref={(el) => (iframeElement = el)}
                    src={buildChallengeUrl(challenge()!.url, params)}
                    width={challenge()!.width}
                    height={challenge()!.height}
                    role="presentation"
                    sandbox="allow-forms allow-popups allow-same-origin allow-scripts allow-top-navigation allow-modals allow-popups-to-escape-sandbox allow-storage-access-by-user-activation"
                  ></iframe>
                </div>
              </Match>
            </Switch>
          </Modal>
        </Show>

        <div class="flex justify-between p-1 bg-gray-200">
          <p class="text-left text-red-400">
            {props.state() === "expired" ? "Verification expired" : ""}
          </p>
          <p class="text-right">Gotcha</p>
        </div>
      </div>
    </div>
  );
}

type Challenge = {
  url: string;
  width: number;
  height: number;
};

async function fetchChallenge(): Promise<Challenge> {
  const origin = new URL(import.meta.url).origin;
  const url = new URL(`${origin}/api/challenge`);

  const response = await fetch(url);
  return (await response.json()) as Challenge;
}

export type ChallengeResponse = {
  token: string;
};

async function processChallenge(
  site_key: string,
  success: boolean,
  challengeUrl: string,
  interactions: Interaction[],
): Promise<string | null> {
  try {
    const origin = new URL(import.meta.url).origin;
    const url = new URL(`${origin}/api/challenge/process`);
    const response = await fetch(url, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        success,
        site_key,
        hostname: window.location.hostname,
        challenge: challengeUrl,
        interactions,
      }),
    });
    if (response.status !== 200)
      throw new Error(
        `processChallenge returned status code ${response.status}`,
      );
    const { token }: ChallengeResponse = await response.json();

    return token;
  } catch (e) {
    return null;
  }
}

function buildChallengeUrl(baseUrl: string, params: SearchParams): string {
  const url = new URL(baseUrl);
  url.searchParams.append("k", params.k);
  url.searchParams.append("hl", params.hl ?? navigator.language);
  url.searchParams.append("theme", params.theme ?? defaultRenderParams.theme!);
  url.searchParams.append("size", params.size ?? defaultRenderParams.size!);
  url.searchParams.append("badge", params.badge ?? defaultRenderParams.badge!);
  url.searchParams.append("sv", params.sv ?? window.location.origin);

  return url.toString();
}
