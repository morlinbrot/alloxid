import { Handlers, PageProps } from "$fresh/server.ts";

import { ServerState } from "routes/_middleware.ts";

import { Layout } from "components";
import AuthForm from "islands/AuthForm.tsx";

export const handler: Handlers = {
  GET(_req, ctx) {
    return ctx.render(ctx.state);
  },
};

export default function Page(props: PageProps<ServerState>) {
  return (
    <Layout state={props.data}>
      <AuthForm mode="Up" />
    </Layout>
  );
}
