export type JsParams = {
  onload?: () => void;
  render?: "explicit";
  hl?: string;
};

export function getJsParams(): JsParams {
  const scripts = document.getElementsByTagName("script");
  const currentScript = scripts[scripts.length - 1];
  const url = new URL(currentScript.src);

  const params: JsParams = {};
  url.searchParams.forEach((value, key) => {
    switch (key) {
      case "onload":
        params.onload = (window as any)[value];
        break;
      case "render":
        params.render = value === "explicit" ? "explicit" : undefined;
        break;
      case "hl":
        params.hl = value;
        break;
    }
  });
  return params;
}
