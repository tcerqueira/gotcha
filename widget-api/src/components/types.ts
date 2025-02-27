import { Accessor } from "solid-js";
import { RenderParams } from "../grecaptcha";
import { LiveState } from "../widget";

export type ChallengeState =
  | "blank"
  | "verifying"
  | "challenging"
  | "verified"
  | "failed"
  | "error";

export type GotchaWidgetProps = RenderParams & {
  liveState: Accessor<LiveState>;
};

export type Challenge = {
  url: string;
  width: number;
  height: number;
};
