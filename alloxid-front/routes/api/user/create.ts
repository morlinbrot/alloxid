import { Handlers } from "$fresh/server.ts";
import { setCookie } from "std/http/cookie.ts";

import { API_URL } from "config";

export const handler: Handlers = {
  async POST(req) {
    const url = new URL(req.url);
    const form = await req.formData();

    const username = String(form.get("username"));
    const password = String(form.get("password"));

    const path = "user";

    console.debug("api/user/create | Creating user with", username, password);
    const res = await fetch(`${API_URL}/${path}`, {
      method: "POST",
      headers: new Headers({ "content-type": "application/json" }),
      body: JSON.stringify({ username, password }),
    });

    if (!res.ok) {
      // TODO: Add some actual error handling.
      console.error(
        "/api/user/index.tsx | Error: ",
        res.status,
        res.statusText,
      );

      // Can't use 500 if we want to redirect to home.
      return new Response(null, {
        status: 303,
        headers: new Headers({ "location": "/" }),
      });
    }

    const { data: { id, token } } = await res.json();

    // if (id && !session) {
    // TODO: A user has been created but not yet confirmed their e-mail address.
    // We could add a flag for the frontend to remind the user.
    // }

    // const exists = await supabase.auth.getUser(String(user));

    // if (exists?.data.user) {
    // TODO: Handle user already existing.
    // }

    const headers = new Headers({ "location": "/" });
    const expires_in = 604800; // 7 days.

    // We're logging the user in right away.
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
