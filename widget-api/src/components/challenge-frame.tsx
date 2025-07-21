import { Interaction, SearchParams } from "@gotcha-widget/lib";
import {
  createEffect,
  createResource,
  createSignal,
  Match,
  Switch,
} from "solid-js";
import { createMediaQuery } from "@solid-primitives/media";
import { defaultRenderParams } from "../grecaptcha";
import CloseSvg from "./icons/close";
import RefreshSvg from "./icons/refresh";
import Logo from "./logo";
import Modal from "./modal";
import { Challenge } from "./types";

type ChallengeFrameProps = {
  open: boolean;
  params: SearchParams;
  onComplete: (response: string) => void;
  onFail: () => void;
  onError: () => void;
  onClose: () => void;
  onReroll?: () => void;
};

export default function ChallengeFrame(props: ChallengeFrameProps) {
  const [iframeRef, setIframeRef] = createSignal<HTMLIFrameElement>();
  createEffect(() => {
    iframeRef()?.focus();
  });

  const [challengeRes, challengeActions] = createResource(
    props.params.k,
    fetchChallenge,
  );

  const isSmallWindow = createMediaQuery("(max-width: 767px)");

  const handleMessage = async (event: MessageEvent) => {
    const challenge = challengeRes();
    if (
      !challenge ||
      event.origin !== new URL(challenge.url).origin ||
      event.source !== iframeRef()?.contentWindow
    ) {
      return;
    }

    const message = event.data;
    switch (message.type) {
      case "response-callback":
        if (!message.success) {
          props.onFail();
          return;
        }
        console.debug(message.interactions);
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

  const onClose = async () => {
    props.onClose();
    await challengeActions.refetch();
  };

  createEffect(() => {
    window.addEventListener("message", handleMessage);
    return () => window.removeEventListener("message", handleMessage);
  });

  return (
    <Modal open={props.open} onClose={onClose}>
      <div class="bg-gray-50 dark:bg-gray-700 border-2 border-gray-400 dark:border-gray-600 rounded-lg p-5">
        <h1 class="text-gray-700 dark:text-gray-50 text-xl text-center mb-4">
          Solve the challenge
        </h1>

        <div
          class={`md:w-[${challengeRes.latest?.width ?? 360}px] md:h-[${challengeRes.latest?.height ?? 500}px]
            w-[${challengeRes.latest?.smallWidth ?? 360}px] h-[${challengeRes.latest?.smallHeight ?? 500}px]`}
        >
          <Switch>
            <Match when={challengeRes.loading}>Loading...</Match>
            <Match when={challengeRes.error}>Something went wrong...</Match>
            <Match when={challengeRes()}>
              <iframe
                ref={setIframeRef}
                src={buildChallengeUrl(challengeRes()!, props.params)}
                width={
                  isSmallWindow()
                    ? challengeRes()!.smallWidth
                    : challengeRes()!.width
                }
                height={
                  isSmallWindow()
                    ? challengeRes()!.smallHeight
                    : challengeRes()!.height
                }
                sandbox="allow-forms allow-scripts allow-same-origin"
              />
            </Match>
          </Switch>
        </div>

        <div class="flex items-center justify-between mt-4">
          <div class="flex gap-4">
            <button
              type="button"
              class="text-gray-400 hover:text-purple-700 dark:hover:text-purple-400"
              onClick={onClose}
            >
              <CloseSvg />
            </button>
            <button
              type="button"
              class="text-gray-400 hover:text-purple-700 dark:hover:text-purple-400"
              onClick={async () => {
                await challengeActions.refetch();
                props.onReroll?.();
              }}
            >
              <RefreshSvg />
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

async function fetchChallenge(siteKey: string): Promise<Challenge | null> {
  try {
    const origin = import.meta.env.VITE_GOTCHA_SV_ORIGIN;
    const url = new URL(`${origin}/api/challenge?site_key=${siteKey}`);

    const response = await fetch(url);
    const challenge_res = await response.json();

    return {
      url: challenge_res.url,
      width: challenge_res.width,
      height: challenge_res.height,
      smallWidth: challenge_res.small_width,
      smallHeight: challenge_res.small_height,
      logoUrl: challenge_res.logo_url,
    };
  } catch (e) {
    console.error("failed to fetch", e);
    return null;
  }
}

type ChallengeResponse = {
  token: string;
};

async function processChallenge(
  siteKey: string,
  success: boolean,
  challengeUrl: string,
  interactions: Interaction[],
): Promise<string | null> {
  try {
    const origin = import.meta.env.VITE_GOTCHA_SV_ORIGIN;
    const url = new URL(`${origin}/api/challenge/process`);
    const response = await fetch(url, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        success,
        site_key: siteKey,
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
    console.error("failed to fetch", e);
    return null;
  }
}

function buildChallengeUrl(challenge: Challenge, params: SearchParams): string {
  const url = new URL(challenge.url);
  url.searchParams.append("k", params.k);
  url.searchParams.append("hl", params.hl ?? navigator.language);
  url.searchParams.append("theme", params.theme ?? defaultRenderParams.theme!);
  url.searchParams.append("size", params.size ?? defaultRenderParams.size!);
  url.searchParams.append("badge", params.badge ?? defaultRenderParams.badge!);
  // TODO: window.location.origin makes no sense
  url.searchParams.append("sv", params.sv ?? window.location.origin);
  if (challenge.logoUrl) {
    url.searchParams.append("logoUrl", challenge.logoUrl);
  }

  return url.toString();
}
