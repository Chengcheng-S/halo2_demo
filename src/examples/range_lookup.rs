use halo2_proofs::{
    circuit::{AssignedCell, Layouter, SimpleFloorPlanner, Value},
    pasta::group::ff::PrimeField,
    plonk::*,
    poly::Rotation,
};

use super::table;

struct ACell<F: PrimeField>(AssignedCell<Assigned<F>, F>);

#[derive(Clone, Debug)]
struct RangeConfig<F: PrimeField, const RANGE: usize, const NUM: usize> {
    value: Column<Advice>,
    table: table::LookupTable<F, RANGE>,
    q_lookup: Selector,
}

impl<F: PrimeField, const RANGE: usize, const NUM: usize> RangeConfig<F, RANGE, NUM> {
    pub fn configure(meta: &mut ConstraintSystem<F>, value: Column<Advice>) -> Self {
        let q_lookup = meta.complex_selector();

        let table = table::LookupTable::<F, RANGE>::configure(meta);

        meta.lookup(|meta| {
            let q_lookup = meta.query_selector(q_lookup);
            let v = meta.query_advice(value, Rotation::cur());
            vec![(q_lookup * v, table.table)]
        });

        RangeConfig {
            value,
            table,
            q_lookup,
        }
    }

    fn assign(
        &self,
        mut layouter: impl Layouter<F>,
        value: [Value<Assigned<F>>; NUM],
    ) -> Result<ACell<F>, Error> {
        layouter.assign_region(
            || "value to check",
            |mut region| {
                self.q_lookup.enable(&mut region, 0)?;
                let mut cell = region
                    .assign_advice(|| "value", self.value, 0, || value[0])
                    .map(ACell);
                for (i, _) in value.iter().enumerate().skip(1) {
                    self.q_lookup.enable(&mut region, i)?;
                    cell = region
                        .assign_advice(|| "value", self.value, i, || value[i])
                        .map(ACell);
                }
                cell
            },
        )
    }
}

#[derive(Debug)]
struct MyCircuit<F: PrimeField, const RANGE: usize, const NUM: usize> {
    value: [Value<Assigned<F>>; NUM],
}
impl<F: PrimeField, const RANGE: usize, const NUM: usize> MyCircuit<F, RANGE, NUM> {
    fn default() -> Self {
        let mut values = vec![];
        for i in 0..NUM {
            values.push(Value::known(Assigned::from(F::from(i as u64))));
        }

        let values = values.try_into().unwrap();
        MyCircuit::<F, RANGE, NUM> { value: values }
    }
}

impl<F: PrimeField, const RANGE: usize, const NUM: usize> Circuit<F> for MyCircuit<F, RANGE, NUM> {
    type Config = RangeConfig<F, RANGE, NUM>;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self::default()
    }

    fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
        let advice = meta.advice_column();

        RangeConfig::<F, RANGE, NUM>::configure(meta, advice)
    }

    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl Layouter<F>,
    ) -> Result<(), Error> {
        config.table.load(layouter.namespace(|| "lookup col"))?;
        config.assign(layouter.namespace(|| "range check"), self.value)?;
        Ok(())
    }
}

mod test {

    #[allow(unused)]
    use super::*;

    #[test]
    fn lookup_example() {
        use halo2_proofs::{dev::MockProver, pasta::Fp};
        const NUM: usize = 3;
        let mut values = vec![];
        for i in 0..NUM {
            values.push(Value::known(Assigned::from(Fp::from(i as u64))));
        }

        let circuit = MyCircuit::<Fp, 16, NUM> {
            value: values.clone().try_into().unwrap(),
        };
        let k = 5;

        let prover = MockProver::run(k, &circuit, vec![]).unwrap();
        prover.assert_satisfied();

        values[1] = Value::known(Assigned::from(Fp::from(18_u64)));
        let circuit = MyCircuit::<Fp, 16, NUM> {
            value: values.clone().try_into().unwrap(),
        };
        let prover = MockProver::run(k, &circuit, vec![]).unwrap();
        assert!(prover.verify().is_err());
    }

    #[cfg(feature = "dev-graph")]
    #[test]
    fn lookup_example_graph() {
        use halo2_proofs::pasta::Fp;
        use plotters::prelude::*;
        let circuit = MyCircuit::<Fp, 16, 3>::default();

        let root =
            BitMapBackend::new("./circuit-layouts/lookup.png", (1024, 3096)).into_drawing_area();
        root.fill(&WHITE).unwrap();
        let root = root.titled("Lookup Layout", ("sans-serif", 60)).unwrap();
        halo2_proofs::dev::CircuitLayout::default()
            // .view_width(0..2)
            // .view_height(0..16)
            .show_labels(true)
            .mark_equality_cells(true)
            .show_equality_constraints(true)
            .render(5, &circuit, &root)
            .unwrap();
    }
}
