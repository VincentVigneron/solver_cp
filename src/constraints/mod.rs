use variables::{VariableError, VariableState, ViewIndex};
use variables::handlers::VariablesHandler;

pub enum ConstraintState {
    Ready,
    NotReady,
}

pub enum PropagationState {
    FixPoint,
    Subsumed,
    NoChange,
}

pub trait Constraint<H: VariablesHandler> {
    fn box_clone(&self) -> Box<Constraint<H>>;
    fn propagate(
        &mut self,
        variables_handler: &mut H,
    ) -> Result<PropagationState, VariableError>;
    fn retrieve_changed_views(
        &self,
        variables_handler: &mut H,
    ) -> Box<Iterator<Item = (ViewIndex, VariableState)>>;
    fn affected_by_changes<'a>(
        &self,
        states: &mut Iterator<Item = &'a (ViewIndex, VariableState)>,
    ) -> bool;
    fn affected_by_change(&self, view_index: &ViewIndex, state: &VariableState) -> bool;
    //fn try_propagate(&mut self, Arc<RefCell<H>>) -> ConstraintState;
}

impl<H: VariablesHandler> Clone for Box<Constraint<H>> {
    fn clone(&self) -> Box<Constraint<H>> {
        self.box_clone()
    }
}

pub trait PropagatorState {}
pub trait Propagator {}

#[macro_use]
pub mod macros;
pub mod arithmetic;
pub mod increasing;
pub mod handlers;
pub mod regular;
pub mod sum;
