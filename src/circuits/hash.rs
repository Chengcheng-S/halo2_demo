use crate::chips::hash::{HashChip, HashConfig};
use halo2_proofs::{
    circuit::{Value, Layouter, SimpleFloorPlanner},
    halo2curves::ff::PrimeField,
    plonk::{Circuit, ConstraintSystem,Error},
};

#[derive(Debug, Default)]
pub struct HashCircuit<F> {
    pub a: Value<F>,
    pub b: Value<F>,
}

impl<F: PrimeField> Circuit<F> for HashCircuit<F> {
    type Config = HashConfig;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self::default()
    }

    fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
        let advice = [
            meta.advice_column(),
            meta.advice_column(),
            meta.advice_column(),
        ];
        let instance = meta.instance_column();
        HashChip::configure(meta, advice, instance)
    }
    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl Layouter<F>,
    ) -> Result<(), Error> {
        let (left, right) = layouter.assign_region(
            || "private inputs",
            |mut region| {
                let left_cell =
                    region.assign_advice(|| "value 1", config.advice[0], 0, || self.a)?;
                let right_cell =
                    region.assign_advice(|| "value 2", config.advice[1], 0, || self.b)?;
                Ok((left_cell, right_cell))
            },
        )?;

        let chip = HashChip::construct(config);
        let hash_result_cell = chip.hash(layouter.namespace(|| "hasher"), left, right)?;

        layouter.constrain_instance(hash_result_cell.cell(), config.instance, 0)
    }
}

#[cfg(test)]
mod tests {
    use super::HashCircuit;

    use halo2_proofs::{circuit::Value, dev::MockProver, halo2curves::pasta::Fp};

    #[test]
    fn test_hash_circuit() {
        let a = 11;
        let b = 6;

        let circuit = HashCircuit {
            a: Value::known(Fp::from(a)),
            b: Value::known(Fp::from(b)),
        };

        let public_input = vec![Fp::from(a * b)];

        let prover = MockProver::run(4, &circuit, vec![public_input.clone()]).unwrap();
        assert!(prover.verify().is_ok());

        let public_inputs2 = vec![Fp::from(a * b + 1)];
        let prover2 = MockProver::run(4, &circuit, vec![public_inputs2.clone()]).unwrap();
        assert!(prover2.verify().is_err());
    }
}
