# fibosquared-halo2
Here's a simple halo2 circuit implementation for squared fibonacci sequence. I've changed it to squared fibonacci sequence for learning purposes.

It's almost the same as an example from the 0xparc learning session:
https://github.com/icemelon/halo2-examples/blob/master/src/fibonacci/example1.rs

My only concern(besides not understanding how the library works more in-depth, which is expected at this moment) is why in the halo2 book example author says that we only have an access to plain multiplication and uses numerical instruction for squaring: https://github.com/zcash/halo2/blob/main/halo2_proofs/examples/simple-example.rs#L185

And in this example for quadriatic residue circuit: https://github.com/scroll-tech/zk-mooc-halo2/blob/main/examples/src/residue_pattern.rs#L54

They simple use square() in the gate implementation (constraint building) and square_root() in "layout" builder - when they populate the cells:
https://github.com/scroll-tech/zk-mooc-halo2/blob/main/examples/src/residue_pattern.rs#L146

EDIT:
Oh I think halo2 book example uses use group::ff::Field; for representing finite field element, but we are using FieldExt and we already have "square()" in FieldExt trait.

So this should be good


Table:
| a  | b | c | selector | input |
| ------------- | ------------- | ------------- | ------------- | ------------- |
| 1 | 1 | 2 | 1 | 1 |
