import Callout from "../../../components/callout";
import HeartIcon from "@heroicons/react/solid/HeartIcon";

# ti`tle`

this is a ~~paragraph~~

> 123321

<div test={{a:{b:[2]}}}>222</div>
231<><div test={true}></div></>

- list item 1
- list item2

```
22
```

    let a = 11;

# Pipelining Package Tasks

In traditional monorepo task runners, like `lerna` or even `yarn`'s own built-in `workspaces run` command, each NPM lifecycle script like `build` or `test` is run [topologically](../glossary#topological-order) (which is the mathematical term for "dependency-first" order) or in parallel individually. Depending on the dependency graph of the monorepo, CPU cores might be left idle—wasting valuable time and resources.

Turborepo gives developers a way to specify task relationships explicitly and conventionally. The advantage here is twofold.

1. Incoming new developers can look at the Turborepo `pipeline` and understand how tasks are related.
2. `turbo` can use this explicit declaration to perform an optimized and scheduled execution based on the abundant availability of multi-core processors.

To give you a sense of how powerful this can be, the below diagram compares the `turbo` vs `lerna` task execution timelines:

![Turborepo vs. Lerna Task Execution](/images/docs/turbo-vs-lerna-execution.png)

Notice that `turbo` is able to schedule tasks efficiently--collapsing waterfalls--whereas `lerna` can only execute one task a time. The results speak for themselves.

## Defining a `pipeline`

To define your project's task dependency graph, use the [`pipeline`](../reference/configuration#pipeline) key in the root `package.json`'s `turbo` configuration. `turbo` interprets this configuration and conventions to properly schedule, execute, and cache the outputs of the tasks in your project.

Each key in the [`pipeline`](../reference/configuration#pipeline) object is the name of a task that can be executed by [`turbo run`](../reference/command-line-reference#turbo-run-task1-task2-1). You can specify its dependencies with the [`dependsOn`](../reference/configuration#dependson) key beneath it as well as some other options related to [caching](./caching).
