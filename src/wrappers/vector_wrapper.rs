use crate::impls;
use crate::{RandomVectorTrait, SequentialMutVectorTrait, SequentialVectorTrait, VectorTrait};

#[derive(Default, Clone)]
pub struct VectorWrapper<V: VectorTrait> {
    pub(crate) object: V,
}

impl<V: VectorTrait> std::ops::Deref for VectorWrapper<V> {
    type Target = V;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.object
    }
}

impl<V: VectorTrait> std::ops::DerefMut for VectorWrapper<V> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.object
    }
}

#[derive(Default, Clone)]
pub struct SequentialVectorWrapper<V: SequentialVectorTrait> {
    pub(crate) object: V,
}

impl<V: SequentialVectorTrait> SequentialVectorWrapper<V> {
    #[inline(always)]
    pub fn rev(&self) -> SequentialVectorWrapper<impl SequentialVectorTrait + '_> {
        SequentialVectorWrapper { object: impls::VectorView::new(self.object.dimension(), self.object.iter().rev()) }
    }
}

impl<V: SequentialVectorTrait> From<V> for SequentialVectorWrapper<V> {
    #[inline(always)]
    fn from(vector_impl: V) -> Self {
        Self { object: vector_impl }
    }
}

impl<V: SequentialVectorTrait> std::ops::Deref for SequentialVectorWrapper<V> {
    type Target = V;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.object
    }
}

impl<V: SequentialVectorTrait> std::ops::DerefMut for SequentialVectorWrapper<V> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.object
    }
}

impl<V: SequentialVectorTrait> std::fmt::Debug for SequentialVectorWrapper<V> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        debug_vector(f, &self.object)
    }
}

impl<V: SequentialMutVectorTrait> SequentialVectorWrapper<V> {
    #[inline(always)]
    pub fn generate_from_iter<I: Iterator<Item = (usize, f64)>>(dimension: usize, nonzero_elements: I) -> Self {
        Self { object: V::generate_from_iter(dimension, nonzero_elements) }
    }

    #[inline(always)]
    pub fn filter<'a>(
        &'a self,
        f: impl Fn(usize, f64) -> bool + Clone + 'a,
    ) -> SequentialVectorWrapper<impl SequentialVectorTrait + 'a> {
        SequentialVectorWrapper {
            object: impls::VectorView::new(self.object.dimension(), self.object.iter().filter(move |&(i, x)| f(i, x))),
        }
    }
}

#[derive(Default, Clone)]
pub struct RandomVectorWrapper<V: RandomVectorTrait> {
    pub(crate) object: V,
}

impl<V: RandomVectorTrait> RandomVectorWrapper<V> {
    pub fn filter<'a>(
        &'a self,
        f: impl Fn(usize, f64) -> bool + Clone + 'a,
    ) -> SequentialVectorWrapper<impl SequentialVectorTrait + 'a> {
        SequentialVectorWrapper {
            object: impls::VectorView::new(self.object.dimension(), self.object.iter().filter(move |(j, x)| f(*j, *x))),
        }
    }
}

impl<V: RandomVectorTrait> std::ops::Deref for RandomVectorWrapper<V> {
    type Target = V;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.object
    }
}

impl<V: RandomVectorTrait> std::ops::DerefMut for RandomVectorWrapper<V> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.object
    }
}

impl<V: RandomVectorTrait> std::fmt::Debug for RandomVectorWrapper<V> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        debug_vector(f, &self.object)
    }
}

impl<V: RandomVectorTrait + SequentialMutVectorTrait> RandomVectorWrapper<V> {
    #[inline(always)]
    pub fn generate_from_iter<I: Iterator<Item = (usize, f64)>>(dimension: usize, nonzero_elements: I) -> Self {
        Self { object: V::generate_from_iter(dimension, nonzero_elements) }
    }
}

/// 次元と同サイズの配列と等価な密ベクトル
pub type DenseVector = RandomVectorWrapper<impls::DenseVector>;

/// 非ゼロ要素の位置を保持する疎ベクトル
pub type SparseVector = RandomVectorWrapper<impls::SparseVector>;

/// 非ゼロ要素と同サイズの配列と等価な疎ベクトル
pub type CompressedVector = SequentialVectorWrapper<impls::CompressedVector>;

/// 単位ベクトル
pub type UnitVector = RandomVectorWrapper<impls::UnitVector>;

impl UnitVector {
    #[inline(always)]
    pub fn new(dimension: usize, nonzero_index: usize) -> Self {
        Self { object: impls::UnitVector::new(dimension, nonzero_index) }
    }
}

fn debug_vector(f: &mut std::fmt::Formatter, vector: &impl SequentialVectorTrait) -> std::fmt::Result {
    write!(f, "{{ dimension = {}, values = [", vector.dimension())?;
    let mut first = true;
    for (index, value) in vector.iter() {
        if first {
            first = false;
        } else {
            write!(f, ", ")?;
        }
        write!(f, "({}, {})", index, value)?;
    }
    write!(f, "] }}")?;
    return Ok(());
}
