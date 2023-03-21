import { Handlers } from "$fresh/server.ts";
import { deleteCookie, getCookies } from "std/http/cookie.ts";

export const handler: Handlers = {
  async GET(req) {
    const url = new URL(req.url);
    const headers = new Headers(req.headers);

    const cookies = getCookies(req.headers);
    const access_token = cookies.auth;

    if (access_token) {
      // await redis.del(access_token);
    }

    deleteCookie(headers, "auth", { path: "/", domain: url.hostname });

    headers.set("location", "/");

    return new Response(null, {
      status: 302,
      headers,
    });
  },
};
