export function loadCss() {
  return new Promise((resolve, reject) => {
    const link = document.createElement("link");
    link.rel = "stylesheet";
    const origin = new URL(import.meta.url).origin;
    const url = new URL(`${origin}/api.css`);
    link.href = url.toString();

    link.onload = resolve;
    link.onerror = () => reject(new Error("CSS failed to load"));

    document.head.appendChild(link);
  });
}
