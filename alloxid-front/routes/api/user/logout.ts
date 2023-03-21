import { Handlers } from "$fresh/server.ts";
import { deleteCookie } from "std/http/cookie.ts";

export const handler: Handlers = {
  GET(req) {
    console.debug("api/user/logout called.");
    const url = new URL(req.url);

    const headers = new Headers();
    headers.set("location", "/");

    deleteCookie(headers, "session_id", { path: "/", domain: url.hostname });
    deleteCookie(headers, "user_id", { path: "/", domain: url.hostname });

    return new Response(null, {
      status: 302,
      headers,
    });
  },
};
