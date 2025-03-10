import { createEffect, createResource, Show } from "solid-js";
import { Challenge } from "./types";
import Modal from "./modal";
import { Interaction, SearchParams } from "@gotcha-widget/lib";
import { defaultRenderParams } from "../grecaptcha";
import Logo from "./logo";

type ChallengeFrameProps = {
  params: SearchParams;
  onComplete: (response: string) => void;
  onError: () => void;
  onClose: () => void;
  onReroll?: () => void;
};

export default function ChallengeFrame(props: ChallengeFrameProps) {
  let iframeElement: HTMLIFrameElement | null = null;
  const [challengeRes, challengeActions] = createResource(
    props.params.k,
    fetchChallenge,
  );

  const handleMessage = async (event: MessageEvent) => {
    const challenge = challengeRes();
    if (
      !challenge ||
      event.origin !== new URL(challenge.url).origin ||
      event.source !== iframeElement?.contentWindow
    ) {
      return;
    }

    const message = event.data;
    switch (message.type) {
      case "response-callback":
        const response = await processChallenge(
          props.params.k,
          message.success,
          challenge.url,
          message.interactions,
        );
        if (response) {
          props.onComplete(response);
        } else {
          props.onError();
        }
        break;
      case "error-callback":
        props.onError();
        break;
    }
  };

  createEffect(() => {
    window.addEventListener("message", handleMessage);
    return () => window.removeEventListener("message", handleMessage);
  });

  return (
    <Modal open={true} onClose={props.onClose}>
      <div class="bg-gray-50 dark:bg-gray-700 rounded-lg p-5 shadow-lg">
        <div class="text-gray-700 dark:text-gray-50 text-xl text-center mb-4">
          Complete the challenge
        </div>

        <Show when={!challengeRes.loading} fallback={"Loading..."}>
          <div class={`w-[${challengeRes()!.width}px]`}>
            <iframe
              ref={(el) => (iframeElement = el)}
              src={buildChallengeUrl(challengeRes()!.url, props.params)}
              width={challengeRes()!.width}
              height={challengeRes()!.height}
              sandbox="allow-forms allow-scripts allow-same-origin"
            />
          </div>
        </Show>

        <div class="flex items-center justify-between mt-4">
          <div class="flex gap-4">
            <button
              type="button"
              class="text-gray-400 hover:text-gray-50"
              onClick={props.onClose}
            >
              <svg class="w-6 h-6" viewBox="0 0 24 24" fill="currentColor">
                <path d="M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z" />
              </svg>
            </button>
            <button
              type="button"
              class="text-gray-400 hover:text-gray-50"
              onClick={async () => {
                await challengeActions.refetch();
                props.onReroll?.();
              }}
            >
              <svg class="w-6 h-6" viewBox="0 0 24 24" fill="currentColor">
                <path d="M17.65 6.35C16.2 4.9 14.21 4 12 4c-4.42 0-7.99 3.58-7.99 8s3.57 8 7.99 8c3.73 0 6.84-2.55 7.73-6h-2.08c-.82 2.33-3.04 4-5.65 4-3.31 0-6-2.69-6-6s2.69-6 6-6c1.66 0 3.14.69 4.22 1.78L13 11h7V4l-2.35 2.35z" />
              </svg>
            </button>
          </div>
          <div class="w-1/4">
            <Logo />
          </div>
        </div>
      </div>
    </Modal>
  );
}

async function fetchChallenge(): Promise<Challenge> {
  const origin = new URL(import.meta.url).origin;
  const url = new URL(`${origin}/api/challenge`);

  const response = await fetch(url);
  return (await response.json()) as Challenge;
}

type ChallengeResponse = {
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
