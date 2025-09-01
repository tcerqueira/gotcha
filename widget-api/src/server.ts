import { Interaction } from "@gotcha-widget/lib";

export type VerificationResponse = {
  success: boolean;
  challenge_ts: string;
  hostname: string | null;
  error_codes: ErrorCodes[] | null;
};

export type ErrorCodes =
  | "missing-input-secret"
  | "invalid-input-secret"
  | "missing-input-response"
  | "invalid-input-response"
  | "bad-request"
  | "timeout-or-duplicate";

export async function siteVerify(
  secret: string,
  token: string,
): Promise<VerificationResponse | null> {
  try {
    const origin = import.meta.env.VITE_GOTCHA_SV_ORIGIN;
    const url = new URL(`${origin}/api/siteverify`);

    const formData = new URLSearchParams();
    formData.append("secret", secret);
    formData.append("response", token);

    const response = await fetch(url, {
      method: "POST",
      headers: {
        "Content-Type": "application/x-www-form-urlencoded",
      },
      body: formData,
    });
    if (response.status !== 200)
      throw new Error(`siteVerify returned status code ${response.status}`);

    return await response.json();
  } catch (e) {
    console.error(e);
    return null;
  }
}

export type FetchChallenge = {
  url: string;
  width: number;
  height: number;
  small_width: number;
  small_height: number;
  logo_url: string | null;
};

export async function fetchChallenge(
  siteKey: string,
): Promise<FetchChallenge | null> {
  try {
    const origin = import.meta.env.VITE_GOTCHA_SV_ORIGIN;
    const url = new URL(`${origin}/api/challenge?site_key=${siteKey}`);

    const response = await fetch(url);
    return await response.json();
  } catch (e) {
    console.error("failed to fetch", e);
    return null;
  }
}

type ChallengeResponse = {
  token: string;
};

export async function processChallenge(
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

export type PowResult = { challenge: string; solution: number };

export type PreAnalysisResponse =
  | { result: "failure" }
  | { result: "success"; response: { token: string } };

export async function processPreAnalysis(
  site_key: string,
  proofOfWork: PowResult,
  interactions: Interaction[],
): Promise<PreAnalysisResponse | null> {
  try {
    const origin = import.meta.env.VITE_GOTCHA_SV_ORIGIN;
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

export async function processAccessibility(
  site_key: string,
  proofOfWork: { challenge: string; solution: number },
): Promise<PreAnalysisResponse | null> {
  try {
    const origin = import.meta.env.VITE_GOTCHA_SV_ORIGIN;
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

export type ProofOfWorkChallenge = {
  token: string;
};

export async function getProofOfWorkChallenge(
  siteKey: string,
): Promise<ProofOfWorkChallenge | null> {
  try {
    const origin = import.meta.env.VITE_GOTCHA_SV_ORIGIN;
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
