import { Interaction } from "@gotcha-widget/lib";
import * as jose from "jose";
import { createEffect } from "solid-js";
import { RenderParams } from "../grecaptcha";
import { PowChallenge, ProofOfWork } from "../proof-of-work";
import { ChallengeState } from "./types";
import Logo from "./logo";

export type PreAnalysisResponse =
  | { result: "failure" }
  | { result: "success"; response: { token: string } };

type ImNotRobotProps = {
  params: RenderParams;
  state: ChallengeState;
  onStateChange: (state: ChallengeState) => void;
  onVerificationComplete: (response: PreAnalysisResponse) => void;
  onError: () => void;
};

export default function ImNotRobot(props: ImNotRobotProps) {
  const interactions: Interaction[] = [];

  createEffect(() => {
    const cleanup = captureInteractions(interactions);
    return cleanup;
  });

  const handleVerification = async (
    verificationFn: (pow: PowResult) => Promise<PreAnalysisResponse | null>,
  ) => {
    if (
      props.state === "verified" ||
      props.state === "verifying" ||
      props.state === "challenging"
    )
      return;

    props.onStateChange("verifying");

    try {
      const powResult = await solveProofOfWork(props.params.sitekey);
      if (!powResult) {
        props.onStateChange("error");
        props.onError();
        return;
      }

      const response = await verificationFn(powResult);
      if (!response) {
        props.onStateChange("error");
        props.onError();
        return;
      }

      props.onVerificationComplete(response);
    } catch (e) {
      props.onStateChange("error");
      props.onError();
    }
  };

  return (
    <div class="rounded-lg w-[304px] h-[68px]">
      <div class="flex justify-between items-stretch space-x-4 h-full">
        <div
          class="pl-6 flex-grow cursor-pointer flex items-center"
          onClick={() =>
            handleVerification((pow) =>
              processPreAnalysis(props.params.sitekey, pow, interactions),
            )
          }
        >
          <span class={`${getTextClass(props.state)}`}>
            {getText(props.state)}
          </span>
        </div>
        <div class="pr-3 flex flex-col justify-evenly items-center max-w-[35%]">
          <Logo />
          <button
            type="button"
            onClick={() =>
              handleVerification((pow) =>
                processAccessibility(props.params.sitekey, pow),
              )
            }
            class="text-purple-500 text-xs self-end hover:underline cursor-pointer"
          >
            Accessibility
          </button>
        </div>
      </div>
    </div>
  );
}

type PowResult = { challenge: string; solution: number };

async function solveProofOfWork(siteKey: string): Promise<PowResult | null> {
  const powChallenge = await getProofOfWorkChallenge(siteKey);
  if (!powChallenge) {
    return null;
  }
  const claims: PowChallenge = jose.decodeJwt(powChallenge.token);
  const solution = await ProofOfWork.solve(claims);
  return { challenge: powChallenge.token, solution };
}

async function processPreAnalysis(
  site_key: string,
  proofOfWork: PowResult,
  interactions: Interaction[],
): Promise<PreAnalysisResponse | null> {
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

async function processAccessibility(
  site_key: string,
  proofOfWork: { challenge: string; solution: number },
): Promise<PreAnalysisResponse | null> {
  try {
    const origin = new URL(import.meta.url).origin;
    const url = new URL(`${origin}/api/challenge/process-accessibility`);
    const response = await fetch(url, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        site_key,
        hostname: window.location.hostname,
        proof_of_work: proofOfWork,
      }),
    });
    if (response.status !== 200)
      throw new Error(
        `processAccessibility returned status code ${response.status}`,
      );

    return await response.json();
  } catch (e) {
    console.error(e);
    return null;
  }
}

type ProofOfWorkChallenge = {
  token: string;
};

async function getProofOfWorkChallenge(
  siteKey: string,
): Promise<ProofOfWorkChallenge | null> {
  try {
    const origin = new URL(import.meta.url).origin;
    const response = await fetch(
      `${origin}/api/challenge/proof-of-work?site_key=${siteKey}`,
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

function getText(state: ChallengeState): string {
  switch (state) {
    case "verified":
      return "Verified!";
    case "failed":
      return "Try again...";
    case "expired":
      return "Expired, verify again.";
    case "error":
      return "Something went wrong.";
    case "verifying":
    case "challenging":
      return "Verifying...";
    default:
      return "I'm not a robot";
  }
}

function getTextClass(state: ChallengeState): string {
  switch (state) {
    case "verified":
      return "text-lg text-green-700 dark:text-green-300";
    case "failed":
    case "expired":
      return "text-md text-red-700 dark:text-red-300";
    case "error":
      return "text-md text-yellow-700 dark:text-yellow-300";
    case "verifying":
    case "challenging":
      return "text-lg text-purple-700 dark:text-purple-300";
    default:
      return "text-lg text-gray-700 dark:text-gray-300";
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
