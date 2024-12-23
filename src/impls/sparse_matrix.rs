use crate::{
    traits::SequentialMutMatrixTrait,
    types::{Direction, COLUMN, ROW, ZERO},
    ColumnMatrixTrait, MatrixTrait, RowMatrixTrait, SequentialMatrixTrait,
};
use fxhash::FxHashMap as HashMap;
use std::{cell::UnsafeCell, ptr::NonNull};

#[derive(Clone, Debug)]
struct Link {
    previous: Option<NonNull<Item>>,
    next: Option<NonNull<Item>>,
}

#[derive(Debug)]
struct Item {
    links: [Link; 2],
    indices: [usize; 2],
    value: f64,
}

#[derive(Debug)]
struct Header {
    first: Option<NonNull<Item>>,
    last: Option<NonNull<Item>>,
    len: usize,
}

impl Default for Header {
    fn default() -> Self {
        Header { first: None, last: None, len: 0 }
    }
}

#[derive(Default, Debug)]
pub struct SparseMatrix {
    hash_map: HashMap<[usize; 2], Box<UnsafeCell<Item>>>,
    headers: [Vec<Header>; 2],
}

impl SequentialMutMatrixTrait for SparseMatrix {
    fn replace_by_iter<I: Iterator<Item = ([usize; 2], f64)>>(&mut self, dimension: [usize; 2], nonzero_elements: I) {
        for d in [ROW, COLUMN] {
            self.headers[d].clear();
            self.headers[d].resize_with(dimension[d], || Header::default());
        }
        self.hash_map.clear();
        for (ij, x) in nonzero_elements {
            self[ij] = x;
        }
    }
}

impl SparseMatrix {
    pub fn remove(&mut self, ij: [usize; 2]) {
        if let Some(item) = self.hash_map.remove(&ij) {
            let pointer = item.get();
            let item = unsafe { &*item.get() };
            for d in [ROW, COLUMN] {
                let index = ij[d];
                let header = &mut self.headers[d][index as usize];
                header.len -= 1;
                match item.links[d].previous {
                    None => {
                        debug_assert!(header.first.as_ref().is_none_or(|x| x.as_ptr() == pointer));
                        header.first = item.links[d].next;
                    }
                    Some(previous) => {
                        let previous_item = unsafe { &mut *previous.as_ptr() };
                        debug_assert!(previous_item.indices[d] == index);
                        debug_assert!(previous_item.links[d].next.as_ref().is_none_or(|x| x.as_ptr() == pointer));
                        previous_item.links[d].next = item.links[d].next;
                    }
                }
                match item.links[d].next {
                    None => {
                        debug_assert!(header.last.as_ref().is_none_or(|x| x.as_ptr() == pointer));
                        header.last = item.links[d].previous;
                    }
                    Some(next) => {
                        let next_item = unsafe { &mut *next.as_ptr() };
                        debug_assert!(next_item.indices[d] == ij[d]);
                        debug_assert!(next_item.links[d].previous.as_ref().is_none_or(|x| x.as_ptr() == pointer));
                        next_item.links[d].previous = item.links[d].previous;
                    }
                }
            }
        }
    }

    pub fn clear_row(&mut self, i: usize) {
        while self.headers[ROW][i].len != 0 {
            let index = {
                let pointer = self.headers[ROW][i].last.unwrap();
                unsafe { &*pointer.as_ptr() }.indices
            };
            debug_assert!(index[ROW] == i);
            self.remove(index);
        }
        debug_assert!(self.headers[ROW][i].first == None);
        debug_assert!(self.headers[ROW][i].last == None);
        debug_assert!(self.headers[ROW][i].len == 0);
    }

    pub fn clear_column(&mut self, j: usize) {
        while self.headers[COLUMN][j].len != 0 {
            let index = {
                let pointer = self.headers[COLUMN][j].last.unwrap();
                unsafe { &*pointer.as_ptr() }.indices
            };
            debug_assert!(index[COLUMN] == j);
            self.remove(index);
        }
        debug_assert!(self.headers[COLUMN][j].first == None);
        debug_assert!(self.headers[COLUMN][j].last == None);
        debug_assert!(self.headers[COLUMN][j].len == 0);
    }

    pub fn zero_clear(&mut self) {
        for d in [ROW, COLUMN] {
            self.headers[d].fill_with(|| Header::default());
        }
        self.hash_map.clear();
    }
}

impl std::ops::Index<[usize; 2]> for SparseMatrix {
    type Output = f64;
    #[inline(always)]
    fn index(&self, index: [usize; 2]) -> &Self::Output {
        match self.hash_map.get(&index) {
            Some(item) => &unsafe { &*item.get() }.value,
            None => &ZERO,
        }
    }
}

impl std::ops::IndexMut<[usize; 2]> for SparseMatrix {
    fn index_mut(&mut self, index: [usize; 2]) -> &mut Self::Output {
        debug_assert!(index[ROW] < self.headers[ROW].len());
        debug_assert!(index[COLUMN] < self.headers[COLUMN].len());
        let item = self.hash_map.entry(index).or_insert_with(|| {
            let item = Box::new(UnsafeCell::new(Item {
                links: [ROW, COLUMN].map(|d| Link { previous: self.headers[d][index[d]].last.clone(), next: None }),
                indices: index,
                value: 0.0,
            }));
            for d in [ROW, COLUMN] {
                let header = &mut self.headers[d][index[d]];
                header.len += 1;
                match header.last {
                    None => {
                        debug_assert!(header.first.is_none());
                        header.first = NonNull::new(item.get());
                    }
                    Some(last) => {
                        unsafe { &mut *last.as_ptr() }.links[d].next = NonNull::new(item.get());
                    }
                }
                header.last = NonNull::new(item.get());
            }
            return item;
        });
        return &mut unsafe { &mut *item.get() }.value;
    }
}

impl MatrixTrait for SparseMatrix {
    #[inline(always)]
    fn dimension(&self) -> [usize; 2] {
        [self.headers[0].len(), self.headers[0].len()]
    }
}

impl SequentialMatrixTrait for SparseMatrix {
    #[inline(always)]
    fn iter(&self) -> impl Iterator<Item = ([usize; 2], f64)> + '_ + Clone {
        self.hash_map.iter().map(|(_, item)| {
            let item = unsafe { &*item.get() };
            ([item.indices[ROW], item.indices[COLUMN]], item.value)
        })
    }
}

impl RowMatrixTrait for SparseMatrix {
    #[inline(always)]
    fn iter_row(&self, i: usize) -> impl DoubleEndedIterator<Item = (usize, f64)> + Clone + '_ {
        Iter::<{ ROW }>::new(&self, i)
    }
}

impl ColumnMatrixTrait for SparseMatrix {
    #[inline(always)]
    fn iter_column(&self, j: usize) -> impl DoubleEndedIterator<Item = (usize, f64)> + Clone + '_ {
        Iter::<{ COLUMN }>::new(&self, j)
    }
}

#[derive(Clone, Debug)]
struct Iter<'a, const D: Direction> {
    sparse_matrix: &'a SparseMatrix,
    index: usize,
    current: Option<NonNull<Item>>,
}

impl<'a, const D: Direction> Iter<'a, D> {
    #[inline(always)]
    fn new(sparse_matrix: &'a SparseMatrix, index: usize) -> Self {
        Self { sparse_matrix: sparse_matrix, index: index, current: None }
    }
}

impl<'a, const D: Direction> std::iter::Iterator for Iter<'a, D> {
    type Item = (usize, f64);
    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        self.current = match self.current {
            None => self.sparse_matrix.headers[D][self.index].first,
            Some(current) => unsafe { &*current.as_ptr() }.links[D].next,
        };
        return self.current.as_ref().map(|&current| {
            let item = unsafe { &*current.as_ptr() };
            debug_assert!(item.indices[D] == self.index);
            (item.indices[!D], item.value)
        });
    }

    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.sparse_matrix.headers[D][self.index].len;
        return (len, Some(len));
    }
}

impl<'a, const D: Direction> std::iter::DoubleEndedIterator for Iter<'a, D> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.current = match self.current {
            None => self.sparse_matrix.headers[D][self.index].last,
            Some(current) => unsafe { &*current.as_ptr() }.links[D].previous,
        };
        return self.current.as_ref().map(|&current| {
            let item = unsafe { &*current.as_ptr() };
            debug_assert!(item.indices[D] == self.index);
            (item.indices[!D], item.value)
        });
    }
}
