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
            Pair setfst(Object newfst) {
                return new Pair(newfst, this.snd);
            }
        }

        class D extends Object {
            Pair left_pair;
            Pair right_pair;
            D(Pair left_pair, Pair right_pair) {
                super();
                this.left_pair=left_pair;
                this.right_pair=right_pair;
            }
            Object leftmost() {
                return this.left_pair.fst;
            }
            Pair left_setfst(Object n) {
                return this.left_pair.setfst(n);
            }
        }
    ").expect("parsing failed");
    println!("AST {:#?}", &ast);
}
