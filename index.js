const js = import("./pkg/compiler")
js.then(js => {
  js.initialize().then(() => console.log(js.transform("\n 1text")))
})
