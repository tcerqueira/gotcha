import { createSignal, createResource, Show, createEffect } from "solid-js";
import { ChallengeState, GotchaWidgetProps, Challenge } from "./types";
import ImNotRobot, { PreAnalysisResponse } from "./im-not-a-robot";
import ChallengeModal from "./challenge-modal";

export function GotchaWidget(props: GotchaWidgetProps) {
  const [state, setState] = createSignal<ChallengeState>("blank");
  const [challenge] = createResource(props.sitekey, fetchChallenge);

  createEffect(() => {
    if (props.liveState() === "expired") {
      setState("expired");
    }
  });

  const handlePreVerificationComplete = (response: PreAnalysisResponse) => {
    if (response.result === "success") {
      setState("verified");
      props.callback?.(response.response.token);
    } else {
      setState("challenging");
    }
  };

  const handleChallengeComplete = (token: string) => {
    setState("verified");
    props.callback?.(token);
  };

  const handleError = () => {
    setState("error");
    props["error-callback"]?.();
  };

  return (
    <div class="gotcha-widget inline-block">
      <div
        class={`border-2 border-gray-300 border-b-4 ${getBorderClass(state())} rounded box-content transition-colors duration-400 ${getBackgroundClass(state())}`}
      >
        <ImNotRobot
          params={props}
          state={state()}
          onStateChange={setState}
          onVerificationComplete={handlePreVerificationComplete}
          onError={handleError}
        />

        <Show when={state() === "challenging"}>
          <ChallengeModal
            challenge={challenge()!}
            params={{
              k: props.sitekey,
              theme: props.theme,
              size: props.size,
              badge: props.badge,
              sv: window.location.origin,
            }}
            onComplete={handleChallengeComplete}
            onError={handleError}
            onClose={() => setState("failed")}
          />
        </Show>
      </div>
    </div>
  );
}

async function fetchChallenge(): Promise<Challenge> {
  const origin = new URL(import.meta.url).origin;
  const url = new URL(`${origin}/api/challenge`);

  const response = await fetch(url);
  return (await response.json()) as Challenge;
}

function getBackgroundClass(state: ChallengeState) {
  switch (state) {
    case "verified":
      return "bg-gradient-to-t from-green-200 to-transparent";
    case "failed":
      return "bg-gradient-to-t from-red-200 to-transparent";
    case "expired":
      return "bg-gradient-to-t from-red-100 to-transparent";
    case "error":
      return "bg-gradient-to-t from-yellow-200 to-transparent";
    case "verifying":
    case "challenging":
      return "bg-gradient-to-t from-purple-200 to-transparent animate-pulse";
    default:
      return "bg-gray-50";
  }
}

function getBorderClass(state: ChallengeState) {
  switch (state) {
    case "verified":
      return "border-b-green-400";
    case "failed":
      return "border-b-red-400";
    case "expired":
      return "border-b-red-300";
    case "error":
      return "border-b-yellow-400";
    case "verifying":
    case "challenging":
      return "border-b-purple-400";
    default:
      return "border-b-gray-300";
  }
}
