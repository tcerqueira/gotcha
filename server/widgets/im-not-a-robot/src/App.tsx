import {
  invokeExpiredCallback,
  invokeResponseCallback,
} from "@gotcha-widget/lib";
import { createMemo, createSignal, type Component } from "solid-js";

function getQueryParam(param: string): string | null {
  const searchParams = new URLSearchParams(window.location.search);
  return searchParams.get(param);
}

type State = "blank" | "verifying" | "verified" | "expired";

const App: Component = () => {
  const [state, setState] = createSignal<State>("blank");
  const checked = createMemo(
    () => state() === "verifying" || state() === "verified",
  );
  const verified = createMemo(() => state() === "verified");
  const expired = createMemo(() => state() === "expired");

  const token = getQueryParam("token");
  if (!token) {
    return <span>Missing api token.</span>;
  }

  const handleCheck = () => {
    setState("verifying");
    // Simulate verification process. Should go to server and receive an encrypted response
    setTimeout(() => {
      setState("verified");
      invokeResponseCallback(true, token);

      setTimeout(() => {
        setState("expired");
        invokeExpiredCallback();
      }, 10000);
    }, 1000);
  };

  return (
    <div class="bg-gray-100 p-6 rounded-lg shadow-md w-screen h-screen">
      <div class="flex items-center space-x-4">
        <div
          class={`w-6 h-6 border-2 rounded cursor-pointer transition-all duration-200 ${
            checked() ? "bg-green-500 border-green-500" : "border-gray-300"
          }`}
          onClick={handleCheck}
        >
          {checked() && (
            <svg
              xmlns="http://www.w3.org/2000/svg"
              class="h-5 w-5 text-white"
              viewBox="0 0 20 20"
              fill="currentColor"
            >
              <path
                fill-rule="evenodd"
                d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z"
                clip-rule="evenodd"
              />
            </svg>
          )}
        </div>
        <span class="text-gray-700">I'm not a robot</span>
      </div>
      {checked() && !verified() && (
        <div class="mt-2 text-sm text-gray-500">Verifying...</div>
      )}
      {verified() && (
        <div class="mt-2 text-sm text-green-500">Verification successful!</div>
      )}
      {expired() && (
        <div class="mt-2 text-sm text-red-500">Verification has expired.</div>
      )}
    </div>
  );
};

export default App;
