import { Interaction } from "@gotcha-widget/lib";
import {
  createMemo,
  createSignal,
  Match,
  onMount,
  Show,
  Switch,
} from "solid-js";
import { RenderParams } from "../grecaptcha";
import { ChallengeResponse } from "./gotcha-widget";

type State = "blank" | "verifying" | "verified" | "failed";

type AnalysisResponse =
  | { result: "failure" }
  | { result: "success"; response: ChallengeResponse };
type ImNotRobotProps = {
  params: RenderParams;
  onResponse: (res: AnalysisResponse) => void;
};

export default function ImNotRobot(props: ImNotRobotProps) {
  const [state, setState] = createSignal<State>("blank");
  const checked = createMemo(
    () => state() === "verifying" || state() === "verified",
  );

  const handleCheck = async () => {
    if (checked()) return;

    setState("verifying");
    let response = await processPreAnalysis(props.params.sitekey, interactions);
    if (!response || response.result === "failure") {
      setState("failed");
      return;
    }

    setState("verified");
    props.onResponse(response);
  };

  const interactions: Interaction[] = [];
  onMount(() => {
    captureInteractions(interactions);
  });

  return (
    <div class="bg-gray-100 p-6 rounded-lg shadow-md w-screen h-screen">
      <div class="flex items-center space-x-4">
        <div
          class={`w-6 h-6 border-2 rounded cursor-pointer transition-all duration-200 ${
            checked() ? "bg-green-500 border-green-500" : "border-gray-300"
          }`}
          onClick={handleCheck}
        >
          <Show when={checked()}>
            <svg
              xmlns="http://www.w3.org/2000/svg"
              class="h-5 w-5 text-white"
              viewBox="0 0 20 20"
              fill="currentColor"
            >
              <path
                fill-rule="evenodd"
                d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z"
                clip-rule="evenodd"
              />
            </svg>
          </Show>
        </div>
        <span class="text-gray-700">I'm not a robot</span>
      </div>
      <Switch>
        <Match when={state() === "verifying"}>
          <div class="mt-2 text-sm text-gray-500">Verifying...</div>
        </Match>
        <Match when={state() === "verified"}>
          <div class="mt-2 text-sm text-green-500">
            Verification successful!
          </div>
        </Match>
        <Match when={state() === "failed"}>
          <div class="mt-2 text-sm text-red-500">Try again...</div>
        </Match>
      </Switch>
    </div>
  );
}

async function processPreAnalysis(
  site_key: string,
  interactions: Interaction[],
): Promise<AnalysisResponse | null> {
  try {
    const origin = new URL(import.meta.url).origin;
    const url = new URL(`${origin}/api/challenge/process-pre-analysis`);
    const response = await fetch(url, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        site_key,
        hostname: window.location.hostname,
        interactions,
      }),
    });
    if (response.status !== 200)
      throw new Error(
        `processPreAnalysis returned status code ${response.status}`,
      );

    return await response.json();
  } catch (e) {
    console.error(e);
    return null;
  }
}

function captureInteractions(interactions: Interaction[]) {
  // Memory leak, fuck this
  document.addEventListener("mousemove", (evt: MouseEvent) => {
    interactions.push({
      ts: Date.now(),
      event: {
        kind: "mousemovement",
        x: evt.offsetX,
        y: evt.offsetY,
      },
    });
  });

  document.addEventListener("mouseup", (evt: MouseEvent) => {
    interactions.push({
      ts: Date.now(),
      event: {
        kind: "mouseclick",
        mouse: "up",
      },
    });
  });

  document.addEventListener("mousedown", (evt: MouseEvent) => {
    interactions.push({
      ts: Date.now(),
      event: {
        kind: "mouseclick",
        mouse: "down",
      },
    });
  });

  document.addEventListener("mouseenter", (evt: MouseEvent) => {
    interactions.push({
      ts: Date.now(),
      event: {
        kind: "mouseenter",
        mouse: "in",
      },
    });
  });

  document.addEventListener("mouseleave", (evt: MouseEvent) => {
    interactions.push({
      ts: Date.now(),
      event: {
        kind: "mouseenter",
        mouse: "out",
      },
    });
  });

  document.addEventListener("keyup", (evt: KeyboardEvent) => {
    interactions.push({
      ts: Date.now(),
      event: {
        kind: "keypress",
        keyMove: "up",
        key: evt.key,
      },
    });
  });

  document.addEventListener("keydown", (evt: KeyboardEvent) => {
    interactions.push({
      ts: Date.now(),
      event: {
        kind: "keypress",
        keyMove: "down",
        key: evt.key,
      },
    });
  });
}
