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
