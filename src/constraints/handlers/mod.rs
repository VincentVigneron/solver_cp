use super::{Constraint, ConstraintBuilder, PropagationState};
use graph::BipartiteGraph;
use std::sync::Arc;
use variables::handlers::VariablesHandler;
use variables::{VariableError, VariableId, VariableState};

pub trait ConstraintsHandlerBuilder<
    Variables: VariablesHandler,
    Constraints: ConstraintsHandler<Variables>,
>
{
    //fn add(&mut self, Box<Constraint<Variables>>);
    fn add(&mut self, Box<ConstraintBuilder<Variables>>);
    fn finalize(self, variables: &mut Variables) -> Result<Constraints, VariableError>;
}

pub trait ConstraintsHandler<Variables: VariablesHandler>: Clone {
    fn propagate_all(
        &mut self,
        variables: &mut Variables,
    ) -> Result<PropagationState, VariableError>;
}

pub struct DefaultConstraintsHandlerBuilder<Variables: VariablesHandler> {
    //constraints: Vec<Box<Constraint<Variables>>>,
    //constraints: Vec<Box<ConstraintBuilder<Variables>>>,
    constraints: Vec<ConstraintBuilder<Variables>>,
}

/*
impl<Variables: VariablesHandler> DefaultConstraintsHandlerBuilder<Variables> {
    pub fn new() -> DefaultConstraintsHandlerBuilder<Variables> {
        DefaultConstraintsHandlerBuilder {
            constraints: Vec::new(),
        }
    }
}

impl<Variables: VariablesHandler>
    ConstraintsHandlerBuilder<Variables, DefaultConstraintsHandler<Variables>>
    for DefaultConstraintsHandlerBuilder<Variables>
{
    //fn add(&mut self, constraint: Box<Constraint<Variables>>) {
    //fn add(&mut self, constraint: Box<ConstraintBuilder<Variables>>) {
    fn add(&mut self, constraint: ConstraintBuilder<Variables>) {
        self.constraints.push(constraint);
    }

    fn finalize(
        mut self,
        variables: &mut Variables,
    ) -> Result<DefaultConstraintsHandler<Variables>, VariableError> {
        //let mut graph: BipartiteGraphBuilder<VariableId, usize, VariableState> =
        //BipartiteGraphBuilder::new();
        //for (idx, constraint) in self.constraints.iter().enumerate() {
        //for (view, state) in constraint.dependencies(&variables) {
        //graph.insert_node1_to_node2(view, state, idx);
        //}
        //}
        let constraints = self.constraints
            .into_iter()
            .map(|cons| finalize(cb, variables))
            //.map(|cons| cons.finalize(variables).map(Box::new))
            .collect::<Result<Vec<_>, VariableError>>()?;
        // Sort according to complexity?
        //for constraint in self.constraints.iter_mut() {
        //constraint.initialise(variables)?;
        //}
        //let len = self.constraints.len();
        let len = constraints.len();
        Ok(DefaultConstraintsHandler {
            //constraints: self.constraints,
            constraints: constraints,
            subsumeds: vec![false; len],
            //graph: Arc::new(graph.finalize()),
            graph: Arc::new(BipartiteGraphBuilder::new().finalize()),
        })
    }
}
*/

#[derive(Clone)]
pub struct DefaultConstraintsHandler<H: VariablesHandler> {
    constraints: Vec<Box<Constraint<H>>>,
    subsumeds: Vec<bool>,
    graph: Arc<BipartiteGraph<VariableId, usize, VariableState>>,
}
unsafe impl<H: VariablesHandler> Sync for DefaultConstraintsHandler<H> {}
unsafe impl<H: VariablesHandler> Send for DefaultConstraintsHandler<H> {}

impl<H: VariablesHandler> ConstraintsHandler<H> for DefaultConstraintsHandler<H> {
    fn propagate_all(
        &mut self,
        variables_handler: &mut H,
    ) -> Result<PropagationState, VariableError> {
        let mut events = self.graph.events();
        for (idx, constraint, subsumed) in self.constraints
            .iter_mut()
            .enumerate()
            .zip(self.subsumeds.iter_mut())
            .map(|((a, b), c)| (a, b, c))
            .filter(|&(_, _, ref subsumed)| !**subsumed)
        {
            constraint.prepare(Box::new(vec![].into_iter()));
            match constraint.propagate(variables_handler)? {
                PropagationState::FixPoint => for (view, state) in constraint.result() {
                    events.add_event(view, idx, state);
                },
                PropagationState::Subsumed => {
                    for (view, state) in constraint.result() {
                        events.add_event(view, idx, state);
                    }
                    *subsumed = true;
                    continue;
                }
                PropagationState::NoChange => {}
            };
        }

        while let Some(iter_events) = events.into_iter() {
            events = self.graph.events();
            for (idx, changes) in iter_events {
                let constraint = self.constraints.get_mut(idx).unwrap();
                let subsumed = self.subsumeds.get_mut(idx).unwrap();
                if *subsumed {
                    continue;
                }
                constraint.prepare(Box::new(changes.into_iter()));
                match constraint.propagate(variables_handler)? {
                    PropagationState::FixPoint => {
                        for (view, state) in constraint.result() {
                            events.add_event(view, idx, state);
                        }
                    }
                    PropagationState::Subsumed => {
                        for (view, state) in constraint.result() {
                            events.add_event(view, idx, state);
                        }
                        *subsumed = true;
                        continue;
                    }
                    PropagationState::NoChange => {}
                };
            }
        }
        Ok(PropagationState::FixPoint)
    }
}
