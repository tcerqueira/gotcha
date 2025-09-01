import { createEffect, createSignal, Show } from "solid-js";
import { PreAnalysisResponse } from "../server";
import ChallengeFrame from "./challenge-frame";
import ImNotRobot from "./im-not-a-robot";
import { ChallengeState, GotchaWidgetProps } from "./types";

export function GotchaWidget(props: GotchaWidgetProps) {
  const [state, setState] = createSignal<ChallengeState>("blank");

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

  const handleFail = () => {
    setState("failed");
  };

  const handleError = () => {
    setState("error");
    props["error-callback"]?.();
  };

  return (
    <div class="gotcha-widget" data-theme={props.theme}>
      <div
        class={`inline-block bg-gray-50 dark:bg-gray-700 transition-colors duration-400 ${getBorderClass(state())}`}
      >
        <div class={`${getBackgroundClass(state())}`}>
          <ImNotRobot
            params={props}
            state={state()}
            onStateChange={setState}
            onVerificationComplete={handlePreVerificationComplete}
            onError={handleError}
          />
          {/* dont run the challenge frame unless we are solving or doing proof of work.
            this way we control prefetching */}
          <Show when={state() === "challenging" || state() === "verifying"}>
            <ChallengeFrame
              open={state() === "challenging"}
              params={{
                k: props.sitekey,
                // TODO: add language support
                hl: null,
                theme: props.theme ?? null,
                size: props.size ?? null,
                badge: props.badge ?? null,
                sv: window.location.origin,
                // TODO: add branding support
                logoUrl: null,
              }}
              onComplete={handleChallengeComplete}
              onFail={handleFail}
              onError={handleError}
              onClose={() => {
                if (state() != "verified" && state() != "error") {
                  setState("failed");
                }
              }}
            />
          </Show>
        </div>
      </div>
    </div>
  );
}

function getBackgroundClass(state: ChallengeState) {
  switch (state) {
    case "verified":
      return `bg-gradient-to-t from-green-300 to-transparent dark:from-green-900/50`;
    case "failed":
      return `bg-gradient-to-t from-red-300 to-transparent dark:from-red-900/50`;
    case "expired":
      return `bg-gradient-to-t from-red-200 to-transparent dark:from-red-900/40`;
    case "error":
      return `bg-gradient-to-t from-yellow-300 to-transparent dark:from-yellow-900/50`;
    case "verifying":
    case "challenging":
      return "bg-gradient-to-t from-purple-300 via-transparent to-transparent dark:from-purple-900 dark:via-transparent dark:to-transparent bg-[size:100%_200%] animate-pulse-gradient";
    default:
      return `bg-transparent`;
  }
}

function getBorderClass(state: ChallengeState) {
  const bColor = (() => {
    switch (state) {
      case "verified":
        return "border-b-green-400 dark:border-b-green-600";
      case "failed":
        return "border-b-red-400 dark:border-b-red-600";
      case "expired":
        return "border-b-red-300 dark:border-b-red-700";
      case "error":
        return "border-b-yellow-400 dark:border-b-yellow-600";
      case "verifying":
      case "challenging":
        return "border-b-purple-400 dark:border-b-purple-600";
      default:
        return "border-b-gray-300 dark:border-b-gray-500";
    }
  })();

  return `border-2 border-gray-300 dark:border-gray-500 border-b-4 ${bColor} rounded`;
}
