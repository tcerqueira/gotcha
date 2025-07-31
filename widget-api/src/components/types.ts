import { Accessor } from "solid-js";
import { RenderParams } from "../gotcha-captcha";
import { LiveState } from "../widget";

export type ChallengeState =
  | "blank"
  | "verifying"
  | "challenging"
  | "verified"
  | "failed"
  | "error"
  | "expired";

export type GotchaWidgetProps = RenderParams & {
  liveState: Accessor<LiveState>;
};

export type Challenge = {
  url: string;
  width: number;
  height: number;
  smallWidth: number;
  smallHeight: number;
  logoUrl: string | null;
};
