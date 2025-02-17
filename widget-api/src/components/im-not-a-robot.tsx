import { Interaction } from "@gotcha-widget/lib";
import {
  createMemo,
  createSignal,
  Match,
  onCleanup,
  onMount,
  Switch,
} from "solid-js";
import * as jose from "jose";
import { RenderParams } from "../grecaptcha";
import { ChallengeResponse } from "./gotcha-widget";
import { PowChallenge, ProofOfWork } from "../proof-of-work";

type State = "blank" | "verified" | "verifying" | "failed";

export type AnalysisResponse =
  | { result: "failure" }
  | { result: "success"; response: ChallengeResponse };
export type ProofOfWorkChallenge = {
  token: string;
};

type ImNotRobotProps = {
  params: RenderParams;
  state: State;
  onResponse: (res: AnalysisResponse) => void;
};

export default function ImNotRobot(props: ImNotRobotProps) {
  const [innerState, setInnerState] = createSignal<State>("blank");
  const checked = createMemo(
    () => innerState() === "verified" || props.state === "verified",
  );
  const verifying = createMemo(
    () => innerState() === "verifying" || props.state === "verifying",
  );

  const handleCheck = async () => {
    if (checked()) return;

    setInnerState("verifying");
    const powChallenge = await getProofOfWorkChallenge(props.params.sitekey);
    if (!powChallenge) {
      setInnerState("blank");
      return;
    }
    const claims: PowChallenge = jose.decodeJwt(powChallenge.token);
    const solution = await ProofOfWork.solve(claims);

    const response = await processPreAnalysis(
      props.params.sitekey,
      { challenge: powChallenge.token, solution },
      interactions,
    );
    if (!response) {
      setInnerState("blank");
      return;
    }

    if (response.result === "success") {
      setInnerState("verified");
    } else {
      setInnerState("blank");
    }
    props.onResponse(response);
  };

  const interactions: Interaction[] = [];
  onMount(() => {
    const cleanup = captureInteractions(interactions);
    onCleanup(cleanup);
  });

  return (
    <div class="bg-gray-100 p-6 rounded-lg shadow-md w-[304px] h-[78px]">
      <div class="flex items-center space-x-4">
        <div
          class={`w-6 aspect-square border-2 rounded cursor-pointer transition-all duration-200 relative ${
            checked() ? "bg-green-500 border-green-500" : "border-gray-300"
          } ${verifying() ? "bg-transparent border-transparent" : ""}`}
          onClick={handleCheck}
        >
          <Switch>
            <Match when={verifying()}>
              <div class="absolute inset-0">
                <div class="animate-spin w-6 h-6 border-2 border-gray-300 border-t-purple-500 rounded-full" />
              </div>
            </Match>
            <Match when={checked()}>
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
            </Match>
          </Switch>
        </div>
        <span class="text-gray-700">I'm not a robot</span>
      </div>
    </div>
  );
}

async function processPreAnalysis(
  site_key: string,
  proofOfWork: { challenge: string; solution: number },
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
        proof_of_work: proofOfWork,
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

async function getProofOfWorkChallenge(
  site_key: string,
): Promise<ProofOfWorkChallenge | null> {
  try {
    const origin = new URL(import.meta.url).origin;
    const response = await fetch(
      `${origin}/api/challenge/proof-of-work?site_key=${site_key}`,
    );
    if (response.status !== 200)
      throw new Error(
        `getProofOfWorkChallenge returned status code ${response.status}`,
      );

    return await response.json();
  } catch (e) {
    console.error(e);
    return null;
  }
}

function captureInteractions(interactions: Interaction[]): () => void {
  const handlers = {
    mousemove: (evt: MouseEvent) => {
      interactions.push({
        ts: Date.now(),
        event: {
          kind: "mousemovement",
          x: evt.offsetX,
          y: evt.offsetY,
        },
      });
    },
    mouseup: (evt: MouseEvent) => {
      interactions.push({
        ts: Date.now(),
        event: {
          kind: "mouseclick",
          mouse: "up",
        },
      });
    },
    mousedown: (evt: MouseEvent) => {
      interactions.push({
        ts: Date.now(),
        event: {
          kind: "mouseclick",
          mouse: "down",
        },
      });
    },
    mouseenter: (evt: MouseEvent) => {
      interactions.push({
        ts: Date.now(),
        event: {
          kind: "mouseenter",
          mouse: "in",
        },
      });
    },
    mouseleave: (evt: MouseEvent) => {
      interactions.push({
        ts: Date.now(),
        event: {
          kind: "mouseenter",
          mouse: "out",
        },
      });
    },
    keyup: (evt: KeyboardEvent) => {
      interactions.push({
        ts: Date.now(),
        event: {
          kind: "keypress",
          keyMove: "up",
          key: evt.key,
        },
      });
    },
    keydown: (evt: KeyboardEvent) => {
      interactions.push({
        ts: Date.now(),
        event: {
          kind: "keypress",
          keyMove: "down",
          key: evt.key,
        },
      });
    },
  };

  Object.entries(handlers).forEach(([event, handler]) => {
    document.addEventListener(
      event as keyof DocumentEventMap,
      handler as EventListener,
    );
  });

  return () => {
    Object.entries(handlers).forEach(([event, handler]) => {
      document.removeEventListener(
        event as keyof DocumentEventMap,
        handler as EventListener,
      );
    });
  };
}
