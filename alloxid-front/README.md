# alloxid-front
Frontend of the `alloxid` family of crates made with [Dioxus](https://github.com/dioxuslabs/dioxus).

The app expects `alloxid-http` to be running at `localhost:3000`.

## Usage
Run with [dioxus-cli](https://crates.io/crates/dioxus-cli):
```
dioxus serve
```

Package with:
```
dioxus build --release
```

## LSDV
"Last Supported dioxus-cli version": 0.3.0
The project was last run on this version of the `dioxus-cli`.

## Learnings
- Dioxus is still very much experimental its documentation is all over the place
- Use the `Dioxus.toml` file to specify properties of the generated `index.html`, like `<title>` and `<link rel="stylesheet">`
- Any custom `index.html` needs to include the following snippet which is not mentioned in the documentation (copied from the generated default file):
```
<script type="module">
  import init from "/./assets/dioxus/{project-name}.js";
  init("/./assets/dioxus/{project-name}_bg.wasm").then(wasm => {
    if (wasm.__wbindgen_start == undefined) {
      wasm.main();
    }
  });
</script>
```
- fermi does not seem to be usable. `use_read` complains about the `Scope` being from the prelude, not core, don't knwo how to fix.
