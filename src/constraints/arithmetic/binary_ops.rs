use constraints::{Constraint, PropagationState};
use std::ops::{Add, Sub};
use variables::{VariableError, VariableState, VariableView, ViewIndex};
use variables::domains::{IterableDomain, PrunableDomain};
use variables::handlers::{SpecificVariablesHandler, VariablesHandler};

#[derive(Clone)]
pub struct AddConstant<Var, View>
where
    View: VariableView + Into<ViewIndex> + 'static,
    View::Variable: PrunableDomain<Type = Var>,
    Var: Eq + Ord + Clone + 'static,
{
    res: View,
    var: View,
    coef: Var,
    output: Option<Vec<(ViewIndex, VariableState)>>,
}

impl<Var, View> AddConstant<Var, View>
where
    View: VariableView + Into<ViewIndex> + 'static,
    View::Variable: PrunableDomain<Type = Var>,
    Var: Eq + Ord + Clone + 'static,
{
    pub fn new(res: View, var: View, coef: Var) -> AddConstant<Var, View> {
        AddConstant {
            res: res,
            var: var,
            coef: coef,
            output: None,
        }
    }
}

use std::fmt::Debug;

impl<Var, View, Handler> Constraint<Handler> for AddConstant<Var, View>
where
    Handler: VariablesHandler + SpecificVariablesHandler<View> + Clone,
    View: VariableView + Into<ViewIndex> + 'static,
    View::Variable: PrunableDomain<Type = Var> + IterableDomain + Debug,
    Var: Eq + Ord + Clone + 'static + Add<Output = Var> + Sub<Output = Var> + Debug,
{
    fn box_clone(&self) -> Box<Constraint<Handler>> {
        let ref_self: &AddConstant<Var, View> = &self;
        let cloned: AddConstant<Var, View> =
            <AddConstant<Var, View> as Clone>::clone(ref_self);

        Box::new(cloned) as Box<Constraint<Handler>>
    }
    fn propagate(
        &mut self,
        variables_handler: &mut Handler,
    ) -> Result<PropagationState, VariableError> {
        let mut output = vec![];
        self.output = None;

        unsafe {
            let res: &mut View::Variable =
                unsafe_from_raw_point!(variables_handler.get_mut(&self.res));
            let var: &mut View::Variable =
                unsafe_from_raw_point!(variables_handler.get_mut(&self.var));
            let domain: Vec<_> = var.iter()
                .cloned()
                .map(|var| var + self.coef.clone())
                .collect();
            let state = res.in_values(domain.into_iter())?;
            match state {
                VariableState::NoChange => {}
                state => {
                    output.push((variables_handler.get_variable_id(&self.res), state));
                }
            }
            let domain: Vec<_> = res.iter()
                .cloned()
                .map(|res| res - self.coef.clone())
                .collect();
            let state = var.in_values(domain.into_iter())?;
            match state {
                VariableState::NoChange => {}
                state => {
                    output.push((variables_handler.get_variable_id(&self.var), state));
                }
            }
        }

        if !output.is_empty() {
            self.output = Some(output);
            Ok(PropagationState::FixPoint)
        } else {
            Ok(PropagationState::NoChange)
        }
    }
    #[allow(unused)]
    fn prepare(&mut self, states: Box<Iterator<Item = ViewIndex>>) {
        // Do nothing
    }
    fn result(&mut self) -> Box<Iterator<Item = (ViewIndex, VariableState)>> {
        use std::mem;
        let mut res = None;
        mem::swap(&mut self.output, &mut res);
        match res {
            None => Box::new(vec![].into_iter()),
            Some(changes) => Box::new(changes.into_iter()),
        }
    }
    #[allow(unused)]
    fn dependencies(
        &self,
        variables: &Handler,
    ) -> Box<Iterator<Item = (ViewIndex, VariableState)>> {
        Box::new(
            vec![
                (
                    variables.get_variable_id(&self.res),
                    VariableState::ValuesChange,
                ),
                (
                    variables.get_variable_id(&self.var),
                    VariableState::ValuesChange,
                ),
            ].into_iter(),
        )
    }
    #[allow(unused)]
    fn initialise(
        &mut self,
        variables_handler: &mut Handler,
    ) -> Result<PropagationState, VariableError> {
        self.propagate(variables_handler)
    }
}