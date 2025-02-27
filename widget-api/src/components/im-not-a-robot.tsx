import { Interaction } from "@gotcha-widget/lib";
import * as jose from "jose";
import { createEffect } from "solid-js";
import { RenderParams } from "../grecaptcha";
import { PowChallenge, ProofOfWork } from "../proof-of-work";
import { CheckboxWithStatus } from "./checkbox-with-status";
import { ChallengeState } from "./types";

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
    if (props.state === "verified") return;

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
    <div class="bg-gray-100 pl-6 pr-1 rounded-lg shadow-md w-[304px] h-[78px]">
      <div class="flex justify-between items-center space-x-4 h-full">
        <CheckboxWithStatus
          state={props.state}
          onClick={() =>
            handleVerification((pow) =>
              processPreAnalysis(props.params.sitekey, pow, interactions),
            )
          }
        />
        <span class="text-gray-700 flex-grow">I'm not a robot</span>
        <div class="flex flex-col justify-evenly items-center self-stretch max-w-[35%] pr-2">
          <img
            src="https://static.wixstatic.com/media/a56dc4_951625a6990f42b6a80975c7beabee2a~mv2.png/v1/fill/w_171,h_38,al_c,q_85,usm_0.66_1.00_0.01,enc_avif,quality_auto/HL_1.png"
            alt="Gotcha logo"
          />
          <span
            onClick={() =>
              handleVerification((pow) =>
                processAccessibility(props.params.sitekey, pow),
              )
            }
            class="text-purple-500 text-xs self-end hover:underline cursor-pointer"
          >
            Accessibility
          </span>
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
