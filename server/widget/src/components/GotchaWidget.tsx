import { Component, createSignal, Show } from "solid-js";

export type GotchaWidgetProps = {
  onSuccess?: (response: string) => void;
  onFailure?: () => void;
};

export function GotchaWidget(props: GotchaWidgetProps) {
  const [isLoading, setLoading] = createSignal(true);

  return (
    <div>
      <Show when={isLoading} fallback={<></>}>
        <p>Loading...</p>
      </Show>
      <span>Are you a robot?</span>
      <button type="button" onClick={props.onFailure}>
        YES
      </button>
      <button
        type="button"
        onClick={() => props.onSuccess && props.onSuccess("congratz")}
      >
        NO
      </button>
    </div>
  );
}
