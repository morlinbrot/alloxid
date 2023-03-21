import { Link } from "components";

type Props = {
  mode: "In" | "Up";
};

export default function AuthForm({ mode }: Props) {
  const signIn = {
    title: "Sign In",
    href: "/user/login",
    text: "Have an account?",
  };

  const signUp = {
    title: "Create account",
    href: "/user/create",
    text: "No account?",
  };

  const buttProps = mode == "In" ? signIn : signUp;
  const footProps = mode == "In" ? signUp : signIn;

  return (
    <>
      <h2>{buttProps.title}</h2>

      <form method="post">
        <input type="email" name="username" autofocus />
        <input type="password" name="password" />

        <button
          type="submit"
          formAction={"/api" + buttProps.href}
        >
          {buttProps.title}
        </button>

        <p>
          {footProps.text} <Link href={footProps.href}>{footProps.title}</Link>
        </p>
      </form>
    </>
  );
}
