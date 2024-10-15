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
  size?: "compact" | "normal";
  tabindex?: number;
  callback?: (token: string) => void;
  "expired-callback"?: () => void;
  "error-callback"?: () => void;
};

export const defaultRenderParams: RenderParams = {
  sitekey: "",
  theme: "light",
  size: "normal",
  tabindex: 0,
};

// TODO: define a secure response token that can be verified server side
export function generateResponseToken(
  success: boolean,
  secret: string,
): string {
  return success ? `${secret}__no-shit-sherlock` : `${secret}__L-bozo`;
}
