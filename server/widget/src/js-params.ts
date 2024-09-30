export type JsParams = {
  onload?: () => void;
  render?: "explicit";
  hl?: string;
};

export function getJsParams(): JsParams {
  const scripts = document.getElementsByTagName("script");
  let url;
  for (let script of scripts) {
    if (!script.src) continue;

    let innerUrl = new URL(script.src);
    if (innerUrl.pathname.includes("/api.js")) {
      url = innerUrl;
      break;
    }
  }

  const params: JsParams = {};
  url?.searchParams.forEach((value, key) => {
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
