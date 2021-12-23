#![allow(dead_code)]

mod ast;
mod parser;

fn main() {
    let ast = parser::parse("
        class A extends Object { A() { super(); } }

        class B extends Object { B() { super(); } }

        class Pair extends Object {
            Object fst;
            Object snd;

            Pair(Object fst, Object snd) {
                super();
                this.fst=fst;
                this.snd=snd;
            }
/*
            Pair setfst(Object newfst) {
                return new Pair(newfst, this.snd);
            }
*/
        }
    ").expect("parsing failed");
    println!("AST {:#?}", &ast);
}
