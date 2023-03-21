import { JSX } from "preact";
import { IS_BROWSER } from "$fresh/runtime.ts";

import { Link } from "components";

export default function LinkButton(
  props: JSX.HTMLAttributes<HTMLAnchorElement>,
) {
  return (
    <Link
      role="button"
      {...props}
      disabled={!IS_BROWSER || props.disabled}
    />
  );
}
