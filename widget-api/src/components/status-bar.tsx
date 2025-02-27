import { Component, Match, Switch } from "solid-js";
import { ChallengeState } from "./types";
import { LiveState } from "../widget";

type StatusBarProps = {
  state: ChallengeState;
  liveState: LiveState;
};

export default function StatusBar(props: StatusBarProps) {
  return (
    <div class="flex justify-between py-1 px-2 bg-gray-200">
      <a href="https://www.gotcha.land" target="_blank">
        <span class="text-purple-500 text-left hover:underline">About us</span>
      </a>
      <StatusMessage state={props.state} liveState={props.liveState} />
    </div>
  );
}

function StatusMessage(props: StatusBarProps) {
  return (
    <p class="text-sm text-right">
      <Switch>
        <Match when={props.state === "verified"}>
          <span class="text-green-500">Verified!</span>
        </Match>
        <Match
          when={props.state === "verifying" || props.state === "challenging"}
        >
          <span class="text-gray-500">Verifying...</span>
        </Match>
        <Match when={props.state === "failed"}>
          <span class="text-red-500">Verification failed.</span>
        </Match>
        <Match when={props.state === "error"}>
          <span class="text-red-500">Oops! Something went wrong...</span>
        </Match>
        <Match when={props.liveState === "expired"}>
          <span class="text-red-500">Verification expired.</span>
        </Match>
      </Switch>
    </p>
  );
}
