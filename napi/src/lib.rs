#![deny(clippy::all)]
#[macro_use]
extern crate napi_derive;
use compiler_core::parse as parse_mdx;

#[napi]
fn parse(source: String) -> String {
  parse_mdx(&source)
}
