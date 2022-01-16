import { compileSync } from "@mdx-js/mdx"
import Benchmark from "benchmark"
import fs from "fs"
import { performance } from "perf_hooks"
import { parse } from "../napi/index.js"
const source = fs.readFileSync("./source.md", "utf8")
const suite = new Benchmark.Suite()

suite
  .add("mdxor napi", function () {
    parse(source)
  })
  .add("mdx", function () {
    compileSync(source)
  })
  .on("cycle", function (event) {
    console.log(String(event.target))
  })
  .run()
