import { Head } from "$fresh/runtime.ts";
import { ComponentChildren } from "preact";

import LinkButton from "islands/LinkButton.tsx";

import { ServerState } from "routes/_middleware.ts";
import { Link } from "components";

type Props = {
  children: ComponentChildren;
  state: ServerState;
};

export function Layout(props: Props) {
  const { user, error } = props.state;

  const buttProps = user
    ? { href: "/api/user/logout", text: "Sign Out" }
    : { href: "/sign-in", text: "Sign In" };

  return (
    <>
      <div class="container">
        <Head>
          <title>alloxid</title>
          <link
            rel="stylesheet"
            href="https://unpkg.com/@picocss/pico@1.*/css/pico.min.css"
          />
          <link rel="stylesheet" href="main.css" />
        </Head>

        <nav>
          <ul>
            <li>
              <a href="/">
                <h1 class="ml-2 text-white">alloxid</h1>
              </a>
            </li>
          </ul>

          <ul>
            <li>
              <Link href="/secret">Secret</Link>
            </li>
          </ul>

          <ul>
            <li>
              <Link href="/sign-up">Create account</Link>
            </li>
            <li>
              <LinkButton href={buttProps.href}>{buttProps.text}</LinkButton>
            </li>
          </ul>
        </nav>
      </div>

      <div class="container">
        {error && (
          <p>There was an error: {`${error.status} ${error.statusText}`}</p>
        )}
        {user && <p>User: {`${user.username || "Unknown"} (${user.id})`}</p>}
        {props.children}
      </div>
    </>
  );
}
