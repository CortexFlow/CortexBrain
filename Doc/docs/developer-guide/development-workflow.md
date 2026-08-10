# Development Workflow

This page groups all the best practices to help newcomers create their first pull request.

## Branch conventions

We review every type of PR that points to the **current development branch**.  
The latest public version is contained in the **0.1.5** branch. The current development version is contained in the **0.1.6** branch.

|Branch | Content|
|-----|------|
|**main**| current production version linked to the latest version |
|**0.1.5**| latest version |
|**0.1.6**| current development version (send your PR here)|
 

## Commit message conventions

- Reference the issue number: `#<n>: <description>` (e.g. _[#199]: updated documentation. Added development-workflow_)

## Contribution flow

Before opening a pull request, you **must** open an issue or a GitHub Discussion for any non-trivial change - pull requests without a related issue or discussion are not accepted (see `CONTRIBUTING.md`). This rule ensures that every update is pre-approved by the maintainers and is aligned with the latest project goals.

- ### "Step 1 - Fork & clone"

    ```bash
    git clone https://github.com/<your-username>/CortexBrain.git
    cd CortexBrain
    git remote add upstream https://github.com/CortexFlow/CortexBrain.git
    ```

- ### "Step 2 - Open an issue or discussion"

    New features and significant changes require a prior [GitHub Discussion](https://github.com/CortexFlow/CortexBrain/discussions). Bug fixes and small improvements can start directly from a [GitHub Issue](https://github.com/CortexFlow/CortexBrain/issues/new/choose). Reference the issue/discussion number in every commit and in the PR description.

- ### "Step 3 - Create a dedicated branch"

    ```bash
    git checkout -b feature/<short-description>
    ```

    Pull requests without a dedicated branch are not accepted. Use a descriptive branch name prefixed by the type of change (`feature/`, `fix/`, `docs/`).

- ### "Step 4 - Commit with a clear message"

    Tag the related issue in every commit using the `#issue: message` convention:

    ```bash
    git commit -m "#78: added TCP connection tracing in conntracker"
    ```

    Keep commits focused. Avoid pull requests larger than ~1500 lines of code unless explicitly justified. (e.g. Grafana example dashboard update)

- ### "Step 5 - Open a Pull Request"

    Open the PR against `CortexFlow/CortexBrain:<current development branch>`. See the **Branch Conventions** to correctly link the PR to the development branch. The PR template asks for:

    - A description of the change
    - The type of change (bug fix, new feature, documentation, refactoring, other)
    - A checklist (tested locally, docs updated, new tests added, builds successfully)
    - Related issues tagged using `#`

    The PR is auto-assigned to a maintainer (`@LorenzoTettamanti`) and an auto-reviewer assignment workflow runs.


## Local build

To build the core components locally, see the [Getting Started for developers](../getting-started/installation.md#getting-started-for-developers) section. The short version:

```bash
# From the repo root
cargo +nightly build --release -p <component-name>
```

Each component also has its own build script under `core/` and `core/src/components/<name>/`.

```bash
# From the repo root CortexBrain/core
cargo +nightly build --release -p <component-name>
```


## Containarization

Every component can be containerized using a prebuilt scripts located in the `CortexBrain/core`.
|Script name| content|
|-----|------|
|agent-api-build.sh| Build the cortexflow-agent container|
|identity-build.sh| Build the identity service container| 
|metrics-build.sh| Build the metrics service container|