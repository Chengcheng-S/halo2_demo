use std::marker::PhantomData;

use halo2_proofs::{circuit::*, pasta::group::ff::PrimeField, plonk::*};

#[derive(Debug, Clone)]
pub(crate) struct LookupTable<F: PrimeField, const RANGE: usize> {
    pub(crate) table: TableColumn,
    _marker: PhantomData<F>,
}

impl<F: PrimeField, const RANGE: usize> LookupTable<F, RANGE> {
    pub fn configure(meta: &mut ConstraintSystem<F>) -> Self {
        let table = meta.lookup_table_column();
        LookupTable {
            table,
            _marker: PhantomData,
        }
    }

    pub fn load(&self, mut layouter: impl Layouter<F>) -> Result<(), Error> {
        layouter.assign_table(
            || "table",
            |mut table| {
                for i in 0..RANGE {
                    table.assign_cell(
                        || "table",
                        self.table,
                        i,
                        || Value::known(F::from(i as u64)),
                    )?;
                }
                Ok(())
            },
        )
    }
}
