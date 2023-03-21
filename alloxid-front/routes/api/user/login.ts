import { Handlers } from "$fresh/server.ts";
import { setCookie } from "std/http/cookie.ts";

import { API_URL } from "config";

export const handler: Handlers = {
  async POST(req) {
    console.debug("api/user/login called.");
    const url = new URL(req.url);
    const form = await req.formData();

    const username = String(form.get("username"));
    const password = String(form.get("password"));

    const path = `${API_URL}/user/login`;
    const res = await fetch(
      path,
      {
        method: "POST",
        headers: new Headers({ "content-type": "application/json" }),
        body: JSON.stringify({ username, password }),
      },
    );

    if (!res.ok) {
      const { status, statusText } = res;
      console.log("api/user/login | ERROR: ", status, statusText);

      return new Response(null, {
        status: 303,
        headers: new Headers({ "location": "/" }),
      });
    }
    // if (error != null || user == null || session == null) {
    // TODO: Add some actual error handling. Differentiate between 500 & 403.
    //   return new Response(null, { status: 500 });
    // }

    const { data: { id, token } } = await res.json();

    // TODO: Let the backend return this.
    const expires_in = 604800; // 7 days.

    const headers = new Headers();
    headers.set("location", "/");

    setCookie(headers, {
      name: "session_id",
      value: token,
      maxAge: expires_in,
      sameSite: "Lax",
      domain: url.hostname,
      path: "/",
      secure: true,
    });

    setCookie(headers, {
      name: "user_id",
      value: id,
      maxAge: expires_in,
      sameSite: "Lax",
      domain: url.hostname,
      path: "/",
      secure: true,
    });

    return new Response(null, {
      status: 303,
      headers,
    });
  },
};
