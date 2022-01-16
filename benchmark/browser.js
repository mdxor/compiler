import { compileSync } from "https://jspm.dev/@mdx-js/mdx@next"
import init, { parse } from "../wasm/pkg/wasm.js"

const suite = new Benchmark.Suite()
const main = async () => {
  await init()
  const res = await fetch("/benchmark/source.md")
  const source = await res.text()
  suite
    .add("mdxor wasm", () => {
      parse(source)
    })
    .add("mdx", () => {
      compileSync(source)
    })
    .on("cycle", event => {
      console.log(String(event.target))
    })
    .run()
}

main()
