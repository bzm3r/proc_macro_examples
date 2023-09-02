// Assume this is where your primary codes liv
// use clap::Parser;
use gen_struct::gen_struct;

gen_struct!(
    #[derive(Parser)]
    struct Foo {
        #[arg(long, env = "BAR")]
        bar: String,
        #[arg(long, env = "BAZ")]
        baz: usize,
    }
);

fn main() {}
