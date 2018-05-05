use graph::Subsumed;
use snowflake::ProcessUniqueId;

pub mod int_var;
pub mod handlers;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VariableState {
    BoundsChange,
    ValuesChange,
    MaxBoundChange,
    MinBoundChange,
    NoChange,
}

impl Subsumed for VariableState {
    fn is_subsumed_under(&self, val: &Self) -> bool {
        match *self {
            VariableState::MaxBoundChange => *val == VariableState::MaxBoundChange,
            VariableState::MinBoundChange => *val == VariableState::MinBoundChange,
            VariableState::BoundsChange => {
                *val != VariableState::ValuesChange && *val != VariableState::NoChange
            }
            VariableState::ValuesChange => *val != VariableState::NoChange,
            VariableState::NoChange => true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VariableError {
    DomainWipeout,
}

#[derive(Hash, Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum IndexType {
    FromVar(usize),
    FromArrayOfVars(usize),
    FromArrayOfVarsVar(usize, usize),
}

pub trait VariableView: Copy {}

#[derive(Hash, Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ViewIndex {
    id: ProcessUniqueId,
    index_type: IndexType,
}

impl ViewIndex {
    pub fn new_from_var(id: ProcessUniqueId, x: usize) -> ViewIndex {
        ViewIndex {
            id: id,
            index_type: IndexType::FromVar(x),
        }
    }

    pub fn new_from_array(id: ProcessUniqueId, x: usize) -> ViewIndex {
        ViewIndex {
            id: id,
            index_type: IndexType::FromArrayOfVars(x),
        }
    }

    pub fn new_from_array_var(id: ProcessUniqueId, x: usize, y: usize) -> ViewIndex {
        ViewIndex {
            id: id,
            index_type: IndexType::FromArrayOfVarsVar(x, y),
        }
    }
    // x sub_view_of x
    // x sub_view_of_y && y sub_view_of x => x == y
    pub fn is_subview_of(&self, idx: &ViewIndex) -> bool {
        if self.id != idx.id {
            return false;
        }
        match self.index_type {
            IndexType::FromArrayOfVarsVar(x, y) => match idx.index_type {
                IndexType::FromArrayOfVars(x_) => x == x_,
                IndexType::FromArrayOfVarsVar(x_, y_) => x == x_ && y == y_,
                _ => false,
            },
            IndexType::FromArrayOfVars(x) => match idx.index_type {
                IndexType::FromArrayOfVars(x_) => x == x_,
                _ => false,
            },
            IndexType::FromVar(x) => match idx.index_type {
                IndexType::FromVar(x_) => x == x_,
                _ => false,
            },
        }
    }

    pub fn get_id(&self) -> ProcessUniqueId {
        self.id.clone()
    }

    pub fn get_type(&self) -> IndexType {
        self.index_type.clone()
    }
}

pub trait AllDisjoint: Iterator<Item = ViewIndex> {
    fn all_disjoint(self) -> Result<(), (ViewIndex, ViewIndex)>
    where
        Self: Sized;
}

// More precise result for all_disjoint (i.e. which views are equal and ,which view is a
// subview of an array)
impl<I> AllDisjoint for I
where
    I: Iterator<Item = ViewIndex>,
{
    fn all_disjoint(self) -> Result<(), (ViewIndex, ViewIndex)>
    where
        Self: Sized,
    {
        use std::iter;
        let views: Vec<_> = self.collect();
        let incompatibles = views
            .iter()
            .enumerate()
            .map(|(i, view)| (view, views.iter().skip(i + 1)))
            .flat_map(|(left, rights)| iter::repeat(left).zip(rights))
            .find(|&(ref left, ref right)| {
                left.is_subview_of(right) || right.is_subview_of(left)
            });
        match incompatibles {
            None => Ok(()),
            Some((left, right)) => Err((left.clone(), right.clone())),
        }
    }
}

/// Trait for types that represent decision variables.
/// A decision variable is variable along side with its domain of allowed values.
/// A variable has to be cloneable because the (tree based) searching process is based on cloning.
pub trait Variable: Clone {
    /// Returns if a variable is affected.
    /// A variable is affected if and only if its a domain is a singleton.
    fn is_affected(&self) -> bool;
    /// Returns the state of a variable without reinitialising it.
    /// The state of a variable describes if and how the domain of the variable has
    /// been updated.
    fn get_state(&self) -> &VariableState;
    /// Returns the state of a variable and reinitialises the state of the
    /// variable. The state of a variable describes if and how the domain of the variable
    /// has been updated.
    fn retrieve_state(&mut self) -> VariableState;
}

/// This trait describes an array of variables. There is two types of array:
/// array of variables and array of references to variables. Both types are manipulated with the
/// same trait. When writting constraints over an array of variables, you should use the Array
/// trait instead of the specific types ArrayOfVars or ArrayOfRefs.
pub trait Array<Var: Variable>: Variable {
    /// Returns a mutable reference to the variable at that position or None if out of bounds.
    fn get_mut(&mut self, position: usize) -> Option<&mut Var>;
    /// Returns a reference to the variable at that position or None if out of bounds.
    fn get(&self, position: usize) -> Option<&Var>;
    /// Returns a mutable reference to the variable at that position without doing bounds check.
    fn get_unchecked_mut(&mut self, position: usize) -> &mut Var;
    /// Returns a reference to the variable at that position without doing bounds check.
    fn get_unchecked(&self, position: usize) -> &Var;
    /// Returns an iterator over the variables.
    fn iter<'array>(&'array self) -> Box<Iterator<Item = &Var> + 'array>;
    /// Returns an iterator that allows modifying each variable.
    fn iter_mut<'array>(&'array mut self) -> Box<Iterator<Item = &mut Var> + 'array>;
    /// Returns the number of variables.
    fn len(&self) -> usize;
}

#[derive(Debug, Clone)]
pub struct ArrayOfVars<Var: Variable> {
    pub variables: Vec<Var>,
    state: VariableState,
}

impl<Var: Variable> ArrayOfVars<Var> {
    pub fn new(len: usize, var: Var) -> Option<Self> {
        Some(ArrayOfVars {
            variables: vec![var.clone(); len],
            state: VariableState::NoChange,
        })
    }
}

impl<Var: Variable> Array<Var> for ArrayOfVars<Var> {
    fn get_mut(&mut self, position: usize) -> Option<&mut Var> {
        self.variables.get_mut(position)
    }

    fn get(&self, position: usize) -> Option<&Var> {
        self.variables.get(position)
    }

    fn get_unchecked_mut(&mut self, position: usize) -> &mut Var {
        unsafe { &mut *(self.variables.get_unchecked_mut(position) as *mut _) }
    }

    fn get_unchecked(&self, position: usize) -> &Var {
        unsafe { self.variables.get_unchecked(position) }
    }

    fn iter<'a>(&'a self) -> Box<Iterator<Item = &Var> + 'a> {
        Box::new(self.variables.iter())
    }

    fn iter_mut<'a>(&'a mut self) -> Box<Iterator<Item = &mut Var> + 'a> {
        Box::new(self.variables.iter_mut())
    }

    fn len(&self) -> usize {
        self.variables.len()
    }
}
impl<Var: Variable> Variable for ArrayOfVars<Var> {
    fn is_affected(&self) -> bool {
        unimplemented!()
    }
    fn get_state(&self) -> &VariableState {
        &self.state
    }
    fn retrieve_state(&mut self) -> VariableState {
        self.variables
            .iter()
            .map(|var| var.get_state())
            .scan(VariableState::NoChange, |acc, state| {
                if *acc == VariableState::BoundsChange {
                    return None;
                }
                *acc = if *acc == VariableState::NoChange {
                    state.clone()
                } else if *state == VariableState::BoundsChange {
                    VariableState::BoundsChange
                } else {
                    acc.clone()
                };

                Some(acc.clone())
            })
            .last()
            .unwrap()
    }
}

#[derive(Debug, Clone)]
pub struct ArrayOfRefs<Var: Variable> {
    pub variables: Vec<*mut Var>,
    state: VariableState,
}

// REF ARRAY BUILDER
impl<Var: Variable> ArrayOfRefs<Var> {
    pub fn new(variables: Vec<*mut Var>) -> Option<Self> {
        Some(ArrayOfRefs {
            variables: variables,
            state: VariableState::NoChange,
        })
    }
}

impl<Var: Variable> Array<Var> for ArrayOfRefs<Var> {
    fn get_mut(&mut self, position: usize) -> Option<&mut Var> {
        unsafe { self.variables.get_mut(position).map(|var| &mut (**var)) }
    }

    fn get(&self, position: usize) -> Option<&Var> {
        unsafe { self.variables.get(position).map(|var| &(**var)) }
    }

    fn get_unchecked_mut(&mut self, position: usize) -> &mut Var {
        unsafe { &mut (**self.variables.get_unchecked_mut(position)) }
    }

    fn get_unchecked(&self, position: usize) -> &Var {
        unsafe { &(**self.variables.get_unchecked(position)) }
    }

    fn iter<'a>(&'a self) -> Box<Iterator<Item = &Var> + 'a> {
        unsafe { Box::new(self.variables.iter().map(|&var| &*var)) }
    }

    fn iter_mut<'a>(&'a mut self) -> Box<Iterator<Item = &mut Var> + 'a> {
        unsafe { Box::new(self.variables.iter_mut().map(|&mut var| &mut *var)) }
    }

    fn len(&self) -> usize {
        self.variables.len()
    }
}

impl<Var: Variable> Variable for ArrayOfRefs<Var> {
    fn is_affected(&self) -> bool {
        unimplemented!()
    }
    fn get_state(&self) -> &VariableState {
        &self.state
    }
    fn retrieve_state(&mut self) -> VariableState {
        self.iter()
            .map(|var| var.get_state())
            .scan(VariableState::NoChange, |acc, state| {
                if *acc == VariableState::BoundsChange {
                    return None;
                }
                *acc = if *acc == VariableState::NoChange {
                    state.clone()
                } else if *state == VariableState::BoundsChange {
                    VariableState::BoundsChange
                } else {
                    acc.clone()
                };

                Some(acc.clone())
            })
            .last()
            .unwrap()
    }
}

#[macro_use]
pub mod macros;
