import { Component, createEffect } from "solid-js";
import { Challenge } from "./types";
import Modal from "./modal";
import { Interaction, SearchParams } from "@gotcha-widget/lib";
import { defaultRenderParams } from "../grecaptcha";

type ChallengeModalProps = {
  challenge: Challenge;
  params: SearchParams;
  onComplete: (response: string) => void;
  onError: () => void;
  onClose: () => void;
};

export default function ChallengeModal(props: ChallengeModalProps) {
  let iframeElement: HTMLIFrameElement | null = null;

  const handleMessage = async (event: MessageEvent) => {
    if (
      !props.challenge ||
      event.origin !== new URL(props.challenge.url).origin ||
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
          props.challenge.url,
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
      <div class={`w-[${props.challenge.width}px]`}>
        <iframe
          ref={(el) => (iframeElement = el)}
          src={buildChallengeUrl(props.challenge.url, props.params)}
          width={props.challenge.width}
          height={props.challenge.height}
          class="border-4 rounded border-purple-200"
          sandbox="allow-forms allow-scripts allow-same-origin"
        />
      </div>
    </Modal>
  );
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
