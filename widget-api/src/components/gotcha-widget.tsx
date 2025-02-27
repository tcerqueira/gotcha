import { createSignal, createResource, Show } from "solid-js";
import { ChallengeState, GotchaWidgetProps, Challenge } from "./types";
import ImNotRobot, { PreAnalysisResponse } from "./im-not-a-robot";
import StatusBar from "./status-bar";
import ChallengeModal from "./challenge-modal";

export function GotchaWidget(props: GotchaWidgetProps) {
  const [state, setState] = createSignal<ChallengeState>("blank");
  const [challenge] = createResource(props.sitekey, fetchChallenge);

  const handlePreVerificationComplete = (response: PreAnalysisResponse) => {
    if (response.result === "success") {
      setState("verified");
      props.callback?.(response.response.token);
    } else {
      // Start challenge flow when pre-verification fails
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
      <div class="border-2 border-purple-200 rounded box-content bg-gray-50">
        <ImNotRobot
          params={props}
          state={state()}
          onStateChange={setState}
          onVerificationComplete={handlePreVerificationComplete}
          onError={handleError}
        />

        <StatusBar state={state()} liveState={props.liveState()} />

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
