export interface WidgetFactory {
  create: () => Widget;
}

export interface Widget {
  render: (container: Element, parameters: RenderParams) => void;
  reset: () => void;
}

export type RenderParams = {
  sitekey: string;
  theme?: "dark" | "light";
  size?: "compact" | "normal" | "invisible";
  badge?: "bottomright" | "bottomleft" | "inline";
  tabindex?: number;
  callback?: (token: string) => void;
  "expired-callback"?: () => void;
  "error-callback"?: () => void;
  isolated?: boolean;
};

export const defaultRenderParams: RenderParams = {
  sitekey: "",
  theme: "light",
  size: "normal",
  badge: "bottomright",
  tabindex: 0,
  isolated: false,
};

export type WidgetMessage =
  | { type: "response-callback"; response: string }
  | { type: "expired-callback" }
  | { type: "error-callback" };

const TARGET_ORIGIN = "*";

export function invokeResponseCallback(
  success: boolean,
  secret: string,
  win: Window = window.parent,
) {
  const message: WidgetMessage = {
    type: "response-callback",
    response: generateResponseToken(success, secret || "not_found"),
  };
  win.postMessage(message, TARGET_ORIGIN);
}

export function invokeExpiredCallback(win: Window = window.parent) {
  const message: WidgetMessage = {
    type: "expired-callback",
  };
  win.postMessage(message, TARGET_ORIGIN);
}

export function invokeErrorCallback(win: Window = window.parent) {
  const message: WidgetMessage = {
    type: "error-callback",
  };
  win.postMessage(message, TARGET_ORIGIN);
}

// TODO: define a secure response token that can be verified server side
function generateResponseToken(success: boolean, secret: string): string {
  return success ? `${secret}__no-shit-sherlock` : `${secret}__L-bozo`;
}
