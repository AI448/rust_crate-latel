#![feature(impl_trait_in_assoc_type)]
#![feature(adt_const_params)]
// #![feature(generic_const_exprs)]

mod impls;
// mod io;
mod operations;
mod traits;
mod types;
mod wrappers;

// pub use io::{random_vector_to_json, sequential_vector_to_json};
// pub use impls::VectorView;
pub use traits::{
    ColumnMatrixTrait, MatrixTrait, MutPermutatorTrait, PermutatorTrait, RandomMutVectorTrait, RandomVectorTrait,
    RowMatrixTrait, SequentialMatrixTrait, SequentialMutMatrixTrait, SequentialMutVectorTrait, SequentialVectorTrait,
    VectorTrait,
};
pub use types::Direction::{self, COLUMN, ROW};
pub use wrappers::{
    BidirectionalMatrix, CCSMatrix, CRCSMatrix, CRSMatrix, ColumnMatrix, CompressedMatrix, CompressedVector,
    DenseVector, DiagonalMatrix, FullPermutator, Permutator, RandomVectorWrapper, RowMatrix, SequentialMatrix,
    SequentialVectorWrapper, SparseMatrix, SparseVector, UnitVector, VectorWrapper,
};

pub use operations::{GenerateFrom, ReplaceBy};

pub trait FMax: Iterator<Item = f64> + Sized {
    fn fmax(self) -> f64 {
        let mut x = -f64::INFINITY;
        for y in self {
            if y.is_nan() {
                return f64::NAN;
            } else {
                x = x.max(y);
            }
        }
        return x;
    }
    fn fmin(self) -> f64 {
        let mut x = f64::INFINITY;
        for y in self {
            if y.is_nan() {
                return f64::NAN;
            } else {
                x = x.min(y);
            }
        }
        return x;
    }
}

impl<I: Iterator<Item = f64>> FMax for I {}
