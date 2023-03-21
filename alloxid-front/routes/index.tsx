import { Handlers, PageProps } from "$fresh/server.ts";

import { ServerState } from "routes/_middleware.ts";

import { Layout, Link } from "components";

export const handler: Handlers = {
  GET(_req, ctx) {
    console.log("routes/index | GET");
    return ctx.render(ctx.state);
  },
};

export default function Home(props: PageProps<ServerState>) {
  const isAllowed = !!props.data.user;

  return (
    <Layout state={props.data}>
      <h2>Welcome</h2>
      <p>
        This is the frontend to the alloxid family of crates demonstrating a
        modern cloud-native stack.
      </p>

      <p>
        <a
          href="https://fresh.deno.dev"
          target="_blank"
          style={{ display: "block", width: "fit-content" }}
        >
          <img
            width="197"
            height="37"
            src="https://fresh.deno.dev/fresh-badge.svg"
            alt="Made with Fresh"
          />
        </a>
      </p>

      <h2>What am I looking at here?</h2>
      <p>Coming soon.</p>

      {isAllowed
        ? <Link href="/api/user/logout">Sign Out</Link>
        : <Link href="/sign-in">Sign In</Link>}
    </Layout>
  );
}
