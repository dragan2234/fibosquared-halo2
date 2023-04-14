use std::{marker::PhantomData};

use halo2_proofs::{
    arithmetic::FieldExt,
    circuit::*,
    plonk::*, poly::Rotation,
    pasta::Fp, dev::MockProver,
};

#[derive(Debug, Clone)]
struct ACell<F: FieldExt>(AssignedCell<F, F>);

#[derive(Debug, Clone)]
struct FiboSquareConfig {
    pub advice: [Column<Advice>; 3],
    pub selector: Selector,
    pub instance: Column<Instance>
}

struct FiboSquareChip<F: FieldExt> {
    config: FiboSquareConfig,
    _marker: PhantomData<F>,
}

impl<F: FieldExt> FiboSquareChip<F> {
    pub fn construct(config: FiboSquareConfig) -> Self {
        Self { config, _marker: PhantomData }
    }

    pub fn configure(
        meta: &mut ConstraintSystem<F>,
        advice: [Column<Advice>; 3],
    ) -> FiboSquareConfig {
        let col_a = advice[0];
        let col_b = advice[1];
        let col_c = advice[2];
        let selector = meta.selector();
        let instance = meta.instance_column();

        meta.enable_equality(col_a);
        meta.enable_equality(col_b);
        meta.enable_equality(col_c);
        meta.enable_equality(instance);

        meta.create_gate("add_squares", |meta| {
            //
            //    col_a       |    col_b        |   col_c      |     selector
            //    a_square        b_square            c                 s
            //
            let s = meta.query_selector(selector);
            let a = meta.query_advice(col_a, Rotation::cur());
            let b = meta.query_advice(col_b, Rotation::cur());
            let c = meta.query_advice(col_c, Rotation::cur());
            vec![s * (a.square() + b.square() - c)] // TODO: check if this is correct or we need additional columns and gate for squaring.
        });

        FiboSquareConfig {
            advice: [col_a, col_b, col_c],
            selector,
            instance,
        }
    }

    pub fn assign_first_row(
        &self,
        mut layouter: impl Layouter<F>,
        a: Option<F>,
        b: Option<F>,
    ) -> Result<(ACell<F>, ACell<F>, ACell<F>), Error>{
        layouter.assign_region(
            || "first row",
            |mut region| {
                self.config.selector.enable(&mut region, 0)?;

                let a_cell = region.assign_advice(
                    || "a_square",
                    self.config.advice[0],
                    0,
                    || a.ok_or(Error::Synthesis),
                ).map(ACell)?;

                let b_cell = region.assign_advice(
                    || "b_square",
                    self.config.advice[1],
                    0,
                    || b.ok_or(Error::Synthesis),
                ).map(ACell)?;

                let c_val = a.and_then(|a| b.map(|b| a + b));

                let c_cell = region.assign_advice(
                    || "c",
                    self.config.advice[2],
                    0,
                    || c_val.ok_or(Error::Synthesis),
                ).map(ACell)?;

                Ok((a_cell, b_cell, c_cell))
        })
    }

    pub fn assign_row(
        &self,
        mut layouter: impl Layouter<F>,
        prev_b: &ACell<F>,
        prev_c: &ACell<F>,
    ) -> Result<ACell<F>, Error> {
        layouter.assign_region(
            || "next row",
            |mut region| {
                self.config.selector.enable(&mut region, 0)?;

                prev_b.0.copy_advice(|| "a", &mut region, self.config.advice[0], 0)?;
                prev_c.0.copy_advice(|| "b", &mut region, self.config.advice[1], 0)?;

                let c_val = prev_b.0.value().and_then(
                    |b| {
                        prev_c.0.value().map(|c| b.square() + c.square())
                    }
                );

                let c_cell = region.assign_advice(
                    || "c",
                    self.config.advice[2],
                    0,
                    || c_val.ok_or(Error::Synthesis),
                ).map(ACell)?;

                Ok(c_cell)
            }
        )
    }

    pub fn expose_public(
        &self,
        mut layouter: impl Layouter<F>,
        cell: &ACell<F>,
        row: usize
    ) -> Result<(), Error> {
        layouter.constrain_instance(cell.0.cell(), self.config.instance, row)
    }
}

#[derive(Default)]
struct MyCircuit<F> {
    pub a: Option<F>,
    pub b: Option<F>,
}

impl<F: FieldExt> Circuit<F> for MyCircuit<F> {
    type Config = FiboSquareConfig;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self::default()
    }

    fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
        let col_a = meta.advice_column();
        let col_b = meta.advice_column();
        let col_c = meta.advice_column();
        FiboSquareChip::configure(meta, [col_a, col_b, col_c])
    }

    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl Layouter<F>,
    ) -> Result<(), Error> {
        let chip = FiboSquareChip::construct(config);

        let (prev_a, mut prev_b, mut prev_c) = chip.assign_first_row(
            layouter.namespace(|| "first row"),
            self.a, self.b,
        )?;

        chip.expose_public(layouter.namespace(|| "private a_squared"), &prev_a, 0)?;
        chip.expose_public(layouter.namespace(|| "private b_squared"), &prev_b, 1)?;

        for _i in 3..5 {
            let c_cell = chip.assign_row(
                layouter.namespace(|| "next row"),
                &prev_b,
                &prev_c,
            )?;
            prev_b = prev_c;
            prev_c = c_cell;
        }

        chip.expose_public(layouter.namespace(|| "output"), &prev_c, 2)?;

        Ok(())
    }
}

fn main() {
    let k = 4;

    let a = Fp::from(1); // F[0]
    let b = Fp::from(1); // F[1]
    let out = Fp::from(29); // F[5]

    let circuit = MyCircuit {
        a: Some(a),
        b: Some(b),
    };

    let public_input = vec![a, b, out];

    let prover = MockProver::run(k, &circuit, vec![public_input.clone()]).unwrap();
    prover.assert_satisfied();
}

