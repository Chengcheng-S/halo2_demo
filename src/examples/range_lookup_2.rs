use super::table2::*;
/// This helper uses a lookup table to check that the value witnessed in a given cell is
/// within a given range.
///
/// The lookup table is tagged by `num_bits` to give a strict range check.
///
/// ------------------
/// | private inputs |
/// ------------------
/// | value |  bit   | q_lookup  | table_n_bits | table_value |
/// -----------------------------------------------------------
/// |  v_0  |   0    |    0      |       1      |      0      |
/// |  v_1  |   1    |    1      |       1      |      1      |
/// |  ...  |  ...   |   1       |       2      |      2      |
/// |  ...  |  ...   |   1       |       2      |      3      |
/// |  ...  |  ...   |   1       |       3      |      4      |
/// |  ...  |  ...   |   1       |       3      |      5      |
/// |  ...  |  ...   |   1       |       3      |      6      |
/// |  ...  |  ...   |   ...     |       3      |      7      |
/// |  ...  |  ...   |   ...     |       4      |      8      |
/// |  ...  |  ...   |   ...     |      ...     |     ...     |
use halo2_proofs::{circuit::*, pasta::group::ff::PrimeField, plonk::*, poly::Rotation};

#[derive(Debug, Clone)]
struct RangeCheckConfig<F: PrimeField, const NUM_BITS: usize, const RANGE: usize> {
    value: Column<Advice>,
    bit: Column<Advice>,
    q_lookup: Selector,
    table: RangeCheckTable<F, NUM_BITS, RANGE>,
}

impl<F: PrimeField, const NUM_BITS: usize, const RANGE: usize>
    RangeCheckConfig<F, NUM_BITS, RANGE>
{
    pub fn configure(meta: &mut ConstraintSystem<F>) -> Self {
        let value = meta.advice_column();
        let bit = meta.advice_column();
        let q_lookup = meta.complex_selector();
        let table = RangeCheckTable::<F, NUM_BITS, RANGE>::configure(meta);

        meta.lookup(|meta| {
            let default_value = Expression::Constant(F::ZERO);
            let default_bit = Expression::Constant(F::ONE);
            let value = meta.query_advice(value, Rotation::cur());
            let bit = meta.query_advice(bit, Rotation::cur());
            let q_lookup = meta.query_selector(q_lookup);
            let non_q = Expression::Constant(F::ONE) - q_lookup.clone();

            let v = value * q_lookup.clone() + non_q.clone() * default_value.clone();
            let b = bit * q_lookup + non_q * default_bit;
            vec![(b, table.n_bits), (v, table.value)]
        });

        RangeCheckConfig {
            value,
            bit,
            q_lookup,
            table,
        }
    }

    fn assign_table(&self, layouter: impl Layouter<F>) -> Result<(), Error> {
        self.table.load(layouter)
    }

    fn assign(
        &self,
        mut layouter: impl Layouter<F>,
        values: &[Value<Assigned<F>>],
        bits: Vec<Value<F>>,
    ) -> Result<(), Error> {
        layouter.assign_region(
            || "bit && vlaue region",
            |mut region| {
                for i in 0..NUM_BITS {
                    self.q_lookup.enable(&mut region, i)?;
                    region.assign_advice(|| "value", self.value, i, || values[i])?;
                    region.assign_advice(|| "bit", self.bit, i, || bits[i])?;
                }
                Ok(())
            },
        )
    }
}

#[derive(Debug, Default)]
struct RangeCheckCircuit<F: PrimeField, const NUM_BITS: usize, const RANGE: usize> {
    bits: Vec<u8>,
    values: Vec<Value<Assigned<F>>>,
}

impl<F: PrimeField, const NUM_BITS: usize, const RANGE: usize> Circuit<F>
    for RangeCheckCircuit<F, NUM_BITS, RANGE>
{
    type Config = RangeCheckConfig<F, NUM_BITS, RANGE>;
    type FloorPlanner = SimpleFloorPlanner;
    fn without_witnesses(&self) -> Self {
        Self::default()
    }
    fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
        RangeCheckConfig::<F, NUM_BITS, RANGE>::configure(meta)
    }
    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl Layouter<F>,
    ) -> Result<(), Error> {
        config.assign_table(layouter.namespace(|| "table"))?;
        let bits = self
            .bits
            .iter()
            .map(|x| Value::known(F::from(*x as u64)))
            .collect::<Vec<Value<F>>>();
        config.assign(layouter.namespace(|| "value"), &self.values, bits)?;
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use halo2_proofs::{dev::MockProver, pasta::Fp};

    use super::*;

    fn circuit() -> RangeCheckCircuit<Fp, 4, 15> {
        const NUM_BITS: usize = 4;
        let mut bits: Vec<u8> = vec![];
        let mut values: Vec<Value<Assigned<Fp>>> = vec![];
        for num_bit in 1u8..=NUM_BITS.try_into().unwrap() {
            for value in 1 << (num_bit - 1)..1 << num_bit {
                println!("value:{:?}, {:?}", num_bit, value);
                values.push(Value::known(Fp::from(value)).into());
                bits.push(num_bit);
            }
        }

        RangeCheckCircuit::<Fp, NUM_BITS, 15> { bits, values }
    }

    #[test]
    fn test_multi_cols_rangecheck_lookup() {
        let k = 5;
        let circuit = circuit();
        let prover = MockProver::run(k, &circuit, vec![]).unwrap();
        prover.assert_satisfied();
    }

    #[cfg(feature = "dev-graph")]
    #[test]
    fn plot_multi_cols_rangecheck_lookup() {
        // Instantiate the circuit with the private inputs.
        let circuit = circuit();
        // Create the area you want to draw on.
        // Use SVGBackend if you want to render to .svg instead.
        use plotters::prelude::*;
        let root = BitMapBackend::new(
            "./circuit-layouts/multi_cols_rangecheck_lookup.png",
            (1024, 768),
        )
        .into_drawing_area();
        root.fill(&WHITE).unwrap();
        let root = root.titled("Lookup2 Circuit", ("sans-serif", 60)).unwrap();

        halo2_proofs::dev::CircuitLayout::default()
            // You can optionally render only a section of the circuit.
            // .view_width(0..2)
            // .view_height(0..16)
            // You can hide labels, which can be useful with smaller areas.
            .show_labels(true)
            // Render the circuit onto your area!
            // The first argument is the size parameter for the circuit.
            .render(5, &circuit, &root)
            .unwrap();
    }
}
