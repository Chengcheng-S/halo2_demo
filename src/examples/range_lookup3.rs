use std::marker::PhantomData;

use halo2_proofs::{
    circuit::{Layouter, SimpleFloorPlanner, Value},
    pasta::group::ff::PrimeField,
    plonk::*,
    poly::Rotation,
};

/// Circuit design:
/// | advice_a| advice_b| q_lookup| table_1 | table_2 |
/// |---------|---------|---------|---------|---------|
/// |    0    |    0    |    1    |    0    |    0    |
/// |    1    |    0    |    1    |    1    |    1    |
/// |    2    |    1    |    1    |    2    |    2    |
/// |    3    |    2    |    1    |    3    |    3    |
/// |         |    3    |    0    |    4    |    4    |
/// |         |         |   ...   |   ...   |   ...   |
/// |         |         |    0    |  RANGE  |  RANGE  |
/// - cur_a ∈ t1
/// - next_b ∈ t2

#[derive(Clone, Debug)]
struct RangeLookupConfig {
    pub advice_a: Column<Advice>,
    pub advice_b: Column<Advice>,
    pub q_lookup: Selector,
    pub table_1: TableColumn,
    pub table_2: TableColumn,
}

struct RangeLookupChip<F: PrimeField> {
    config: RangeLookupConfig,
    _marker: PhantomData<F>,
}

impl<F: PrimeField> RangeLookupChip<F> {
    fn construct(config: RangeLookupConfig) -> Self {
        Self {
            config,
            _marker: PhantomData,
        }
    }

    fn configure(meta: &mut ConstraintSystem<F>) -> RangeLookupConfig {
        let advice_a = meta.advice_column();
        let advice_b = meta.advice_column();
        let q_lookup = meta.complex_selector();
        let table_1 = meta.lookup_table_column();
        let table_2 = meta.lookup_table_column();

        meta.enable_equality(advice_a);
        meta.enable_equality(advice_b);

        meta.lookup(|meta| {
            let cur_a = meta.query_advice(advice_a, Rotation::cur());
            let next_b = meta.query_advice(advice_b, Rotation::next());
            let s = meta.query_selector(q_lookup);

            //  (0,0) ---> table_1, table_2
            vec![(s.clone() * cur_a, table_1), (s * next_b, table_2)]
        });

        RangeLookupConfig {
            advice_a,
            advice_b,
            q_lookup,
            table_1,
            table_2,
        }
    }

    fn assign(
        &self,
        mut layouter: impl Layouter<F>,
        a_values: &[Value<F>],
        b_values: &[Value<F>],
    ) -> Result<(), Error> {
        let _ = layouter.assign_region(
            || "",
            |mut region| {
                for (i, _) in a_values.iter().enumerate() {
                    self.config.q_lookup.enable(&mut region, i)?;
                    region.assign_advice(|| "a col", self.config.advice_a, i, || a_values[i])?;
                }

                for (i, _) in b_values.iter().enumerate() {
                    region.assign_advice(|| "b col", self.config.advice_b, i, || b_values[i])?;
                }

                Ok(())
            },
        );
        let _ = layouter.assign_table(
            || "lookup table",
            |mut region| {
                for i in 0..10 {
                    region.assign_cell(
                        || "table_1",
                        self.config.table_1,
                        i,
                        || Value::known(F::from(i as u64)),
                    )?;
                    region.assign_cell(
                        || "table_2",
                        self.config.table_2,
                        i,
                        || Value::known(F::from(i as u64)),
                    )?;
                }
                Ok(())
            },
        );
        Ok(())
    }
}

#[derive(Default)]
struct RangeLookupCircuit<F: PrimeField> {
    a: Vec<Value<F>>,
    b: Vec<Value<F>>,
}

impl<F: PrimeField> Circuit<F> for RangeLookupCircuit<F> {
    type Config = RangeLookupConfig;
    type FloorPlanner = SimpleFloorPlanner;
    fn without_witnesses(&self) -> Self {
        Self::default()
    }
    fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
        RangeLookupChip::configure(meta)
    }
    fn synthesize(&self, config: Self::Config, layouter: impl Layouter<F>) -> Result<(), Error> {
        let chip = RangeLookupChip::construct(config);
        chip.assign(layouter, &self.a, &self.b)?;
        Ok(())
    }
}

mod test {

    use super::*;
    use halo2_proofs::pasta::Fp;

    #[allow(unused)]
    fn mycircuit() -> RangeLookupCircuit<Fp> {
        let a = [0, 1, 2, 3, 4];
        let b = [0, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
        println!("a: {:?} ", a);
        println!("b: {:?} ", b);
        let a = a.map(|v| Value::known(Fp::from(v))).to_vec();
        let b = b.map(|v| Value::known(Fp::from(v))).to_vec();

        RangeLookupCircuit::<Fp> { a, b }
    }

    #[test]
    fn test_range_lookup() {
        let k = 5;
        let circuit = mycircuit();
        let prover = halo2_proofs::dev::MockProver::run(k, &circuit, vec![]).unwrap();
        prover.assert_satisfied();
    }

    #[cfg(feature = "dev-graph")]
    #[test]
    fn draw_range_lookup() {
        use plotters::prelude::*;
        let root = BitMapBackend::new("./circuit-layouts/range_lookup3.png", (1024, 768))
            .into_drawing_area();
        root.fill(&WHITE).unwrap();
        let root = root.titled("Range Lookup", ("sans-serif", 60)).unwrap();
        let circuit = mycircuit();

        halo2_proofs::dev::CircuitLayout::default()
            .show_labels(true)
            .mark_equality_cells(true)
            .show_equality_constraints(true)
            // Render the circuit onto your area!
            // The first argument is the size parameter for the circuit.
            .render(5, &circuit, &root)
            .unwrap();
    }
}
