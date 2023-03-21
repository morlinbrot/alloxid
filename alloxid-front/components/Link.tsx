import { JSX } from "preact";
// import { IS_BROWSER } from "$fresh/runtime.ts";

export function Link(props: JSX.HTMLAttributes<HTMLAnchorElement>) {
  return (
    <a
      {...props}
      // disabled={!IS_BROWSER || props.disabled}
      disabled={props.disabled}
      class={`${props.class ?? ""}`}
    />
  );
}
