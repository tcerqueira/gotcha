import { SearchParams, WidgetMessage, Interaction } from "@gotcha-widget/lib";
import {
  Accessor,
  createMemo,
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
import { LiveState } from "../widget";

type AdditionalParams = {
  liveState: Accessor<LiveState>;
};
export type GotchaWidgetProps = RenderParams & AdditionalParams;

type State = "live" | "verifying" | "verified" | "failed" | "error";

export function GotchaWidget(props: GotchaWidgetProps) {
  let iframeElement: HTMLIFrameElement | null = null;
  const [state, setState] = createSignal<State>("live");
  const [challenge] = createResource(props.sitekey, fetchChallenge);
  const showChallenge = createMemo(() => state() === "verifying");

  const handlePreChallengeResponse = (response: AnalysisResponse) => {
    if (response.result === "success") {
      setState("verified");
      props.callback?.(response.response.token);
    } else {
      setState("verifying");
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
          setState(message.success ? "verified" : "failed");
          props.callback?.(response);
        } else {
          setState("error");
          props["error-callback"]?.();
        }
        break;
      case "error-callback":
        setState("error");
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

  const params: SearchParams = {
    k: props.sitekey,
    theme: props.theme,
    size: props.size,
    badge: props.badge,
    sv: window.location.origin,
  };

  const notRobotState = createMemo(() => {
    switch (state()) {
      case "verified":
        return "verified";
      case "verifying":
        return "verifying";
      default:
        return "blank";
    }
  });

  return (
    <div class="gotcha-widget inline-block">
      <div class={`border-2 border-purple-200 rounded box-content bg-gray-50`}>
        <ImNotRobot
          params={props}
          state={notRobotState()}
          onResponse={handlePreChallengeResponse}
        />
        <div class="flex justify-between p-1 bg-gray-200">
          <p class="text-left">
            <Switch>
              <Match when={state() === "verified"}>
                <span class="text-green-400">Verified!</span>
              </Match>
              <Match when={state() === "verifying"}>
                <span class="text-gray-500">Verifiying...</span>
              </Match>
              <Match when={state() === "failed"}>
                <span class="text-red-400">Verification failed.</span>
              </Match>
              <Match when={state() === "error"}>
                <span class="text-red-400">Oops! Something went wrong...</span>
              </Match>
              <Match when={props.liveState() === "expired"}>
                <span class="text-red-400">Verification expired.</span>
              </Match>
            </Switch>
          </p>
          <p class="text-right">Gotcha</p>
        </div>

        <Show when={showChallenge()}>
          <Modal open={showChallenge()} onClose={() => setState("failed")}>
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
                    class="border-4 rounded border-purple-200 overflow-hidden m-0 p-0 focus-visible:outline-none"
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
