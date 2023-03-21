import { MiddlewareHandlerContext } from "$fresh/server.ts";
import { getCookies } from "std/http/cookie.ts";

import { API_URL } from "config";

export type User = {
  id: string;
  username: string;
};

export type ServerState = {
  user: User | null;
  error?: { status: number; statusText: string };
};

export async function handler(
  req: Request,
  ctx: MiddlewareHandlerContext<ServerState>,
) {
  const url = new URL(req.url);
  const cookies = getCookies(req.headers);
  const access_token = cookies.session_id;
  const user_id = cookies.user_id;
  console.debug(`_middle | ${req.method} ${url}`);

  const protected_route = url.pathname == "/secret";

  const headers = new Headers();
  headers.set("location", "/");

  if (protected_route && !access_token) {
    // Can't use 403 if we want to redirect to home page.
    console.debug(
      "_middle | Tried to access protected route without token, redirecting.",
    );
    return new Response(null, { headers, status: 303 });
  }

  if (access_token && user_id) {
    const headers = new Headers({ authorization: `Bearer ${access_token}` });

    try {
      const res = await fetch(`${API_URL}/user/${user_id}`, {
        headers,
      });

      if (!res.ok) {
        const { status, statusText } = res;
        console.error("_middle | ERROR: ", status, statusText);
        ctx.state.error = { status, statusText };
        return await ctx.next();
      }

      const json = await res.json();
      const { id, username } = json.data;
      console.debug(`_middle | Fetched user id={${id}}`);

      ctx.state.user = { id, username };
    } catch (err) {
      console.error("_middle | ERROR: ", err);
      if (
        err instanceof TypeError && err.message.includes("refused")
      ) {
        // console.debug("IN HERE: ", err.message);
        ctx.state.error = {
          status: 521,
          statusText: "Connection refused. Is the backend running?",
        };
      }
    }
  }

  return await ctx.next();
}
