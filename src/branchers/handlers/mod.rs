use std::marker::PhantomData;
//use variables::ViewIndex;
//use variables::handlers::{get_from_handler, get_mut_from_handler,
//SpecificVariablesHandler, VariablesHandler, ViewIndex};
use variables::ViewIndex;
use variables::handlers::VariablesHandler;
//use variables::int_var::*;

pub trait VariableSelector<Handler>
where
    Handler: VariablesHandler,
{
    fn select(&mut self, variables: &Handler) -> Option<ViewIndex>;
}

pub trait ValuesSelector<Handler>
where
    Handler: VariablesHandler,
{
    fn select(
        &mut self,
        variables: &Handler,
        view: ViewIndex,
    ) -> Option<Box<Iterator<Item = Box<Fn(&mut Handler) -> ()>>>>;
}

pub trait Brancher<Handler>
where
    Handler: VariablesHandler,
{
    fn branch(&mut self, variables: &mut Handler) -> Result<(), ()>;
}

pub struct DefaultBrancher<Handler, VarSel, ValSel>
where
    Handler: VariablesHandler,
    VarSel: VariableSelector<Handler>,
    ValSel: ValuesSelector<Handler>,
{
    variables_selector: VarSel,
    values_selector: ValSel,
    phantom: PhantomData<Handler>,
}

impl<Handler, VarSel, ValSel> Brancher<Handler>
    for DefaultBrancher<Handler, VarSel, ValSel>
where
    Handler: VariablesHandler,
    VarSel: VariableSelector<Handler>,
    ValSel: ValuesSelector<Handler>,
{
    fn branch(&mut self, variables: &mut Handler) -> Result<(), ()> {
        let variable = self.variables_selector.select(&variables);
        match variable {
            Some(idx) => {
                let values = self.values_selector.select(&variables, idx);
                match values {
                    Some(values) => {
                        //Ok(values)
                        Ok(())
                    }
                    None => Err(()),
                }
            }
            None => Err(()),
        }
    }
}

/*
pub trait SelectorState {}

pub trait SelectVariable<Handler: VariablesHandler, State: SelectorState> {
    fn select(
        &self,
        variables: &Handler,
        state: &State,
    ) -> Option<Box<FnOnce() -> Iterator<Item = (State, Handler)>>>;
}
*/

/*
pub trait Brancher<Handler: VariablesHandler> {
    fn box_clone(&self) -> Box<Brancher<Handler>>;
    fn branch(&mut self, variables: &Handler) -> Option<Handler>;
    fn branch_fn(
        &self,
        variables: &Handler,
    ) -> Option<Box<Iterator<Item = Box<Fn(&mut Handler) -> ()>>>>;
}

pub trait BranchersHandler<Handler: VariablesHandler>: Clone {
    fn branch(&mut self, variables: &Handler) -> Option<(Self, Handler)>;
    fn branch_fn(
        &self,
        variables: &Handler,
    ) -> Option<Box<Iterator<Item = Box<Fn(&mut Handler) -> ()>>>>;
}

impl<Handler: VariablesHandler> Clone for Box<Brancher<Handler>> {
    fn clone(&self) -> Box<Brancher<Handler>> {
        self.box_clone()
    }
}

#[derive(Clone)]
pub struct MultipleBrancherHandler<Handler: VariablesHandler> {
    branchers: Vec<Box<Brancher<Handler>>>,
}

impl<Handler: VariablesHandler> BranchersHandler<Handler>
    for MultipleBrancherHandler<Handler> {
    fn branch(
        &mut self,
        variables: &Handler,
    ) -> Option<(MultipleBrancherHandler<Handler>, Handler)> {
        // change to avoid duplication of the next used brancher
        //let mut new_brancher = self.clone();
        //let next_branch = self.branchers
        //.iter_mut()
        //.map(|brancher| brancher.branch(&variables))
        //.enumerate()
        //.find(|&(_, ref branch)| branch.is_some());
        //if let Some((idx, branch)) = next_branch {
        //let (branch, variables) = branch.unwrap();
        //new_brancher.branchers[idx] = branch;
        //return Some((Box::new(new_brancher), variables));
        //}
        //None
        unimplemented!()
    }
    fn branch_fn(
        &self,
        variables: &Handler,
    ) -> Option<Box<Iterator<Item = Box<Fn(&mut Handler) -> ()>>>> {
        self.branchers
            .iter()
            .map(|brancher| brancher.branch_fn(&variables))
            .find(|branch| branch.is_some())
            .map(|branch| branch.expect("specific branch"))
    }
}

impl<Handler: VariablesHandler> MultipleBrancherHandler<Handler> {
    pub fn new() -> MultipleBrancherHandler<Handler> {
        MultipleBrancherHandler {
            branchers: Vec::new(),
        }
    }

    pub fn add_brancher(&mut self, brancher: Box<Brancher<Handler>>) -> () {
        self.branchers.push(brancher);
    }
}

#[derive(Clone)]
pub struct FirstVariableBrancher<View>
where
    View: ViewIndex,
{
    //views: Vec<(bool, View)>,
    views: Vec<View>,
}

impl<View> FirstVariableBrancher<View>
where
    View: ViewIndex,
{
    pub fn new(views: Vec<View>) -> FirstVariableBrancher<View> {
        FirstVariableBrancher {
            //views: views.into_iter().map(|view| (false, view)).collect(),
            views: views,
        }
    }
}

// MAYBE branch generate iterator or Not?
// MAYBE Clone variable inside Iterator?
// MAYBE Store immutable reference inside it?
// MAYBE New consuming var? (ie no cloning during search)
impl<Handler, View> Brancher<Handler> for FirstVariableBrancher<View>
where
    Handler: VariablesHandler + SpecificVariablesHandler<IntVar, View> + Clone + 'static,
    View: Into<ViewIndex> + Clone + 'static,
{
    fn box_clone(&self) -> Box<Brancher<Handler>> {
        let ref_self: &FirstVariableBrancher<_> = &self;
        let cloned: FirstVariableBrancher<_> =
            <FirstVariableBrancher<_> as Clone>::clone(ref_self);

        Box::new(cloned) as Box<Brancher<Handler>>
    }

    fn branch(&mut self, variables: &Handler) -> Option<Handler> {
        unimplemented!()
    }

    fn branch_fn(
        &self,
        variables: &Handler,
    ) -> Option<Box<Iterator<Item = Box<Fn(&mut Handler) -> ()>>>> {
        let idx = self.views
            .iter()
            .position(|ref view| !get_from_handler(variables, &view).is_fixed());
        match idx {
            Some(idx) => {
                let view = self.views[idx].clone();
                let values = get_from_handler(variables, &view).domain_iter();
                //self.views.drain(0..(idx + 1));
                Some(Box::new(FirstVariableBrancherIterator::new(view, values)))
                //Some(Box::new(move |vars| {
                //let var = get_mut_from_handler(vars, &view);
                //var.unsafe_set_value(min);
                //}))
            }
            None => None,
        }
        //unimplemented!()
    }
}

pub struct FirstVariableBrancherIterator<View, Handler>
where
    View: ViewIndex,
    Handler: VariablesHandler + SpecificVariablesHandler<IntVar, View>,
{
    view: View,
    values: IntVarDomainIterator,
    phantom: PhantomData<Handler>,
}

impl<View, Handler> FirstVariableBrancherIterator<View, Handler>
where
    View: ViewIndex,
    Handler: VariablesHandler + SpecificVariablesHandler<IntVar, View>,
{
    fn new(
        view: View,
        values: IntVarDomainIterator,
    ) -> FirstVariableBrancherIterator<View, Handler> {
        FirstVariableBrancherIterator {
            view: view,
            values: values,
            phantom: PhantomData,
        }
    }
}

impl<Handler, View> Iterator for FirstVariableBrancherIterator<View, Handler>
where
    Handler: VariablesHandler + SpecificVariablesHandler<IntVar, View>,
    View: Into<ViewIndex> + 'static,
{
    type Item = Box<Fn(&mut Handler) -> ()>;

    fn next(&mut self) -> Option<Box<Fn(&mut Handler) -> ()>> {
        match self.values.next() {
            Some(value) => {
                let view = self.view.clone();
                Some(Box::new(move |vars| {
                    let var: &mut IntVar = get_mut_from_handler(vars, &view);
                    unsafe {
                        var.unsafe_set_value(value);
                    }
                }))
            }
            _ => None,
        }
        //}))
        //self.values.next().and_then(|value| {
        //let view = self.view.clone();
        //let value = value;
        //Some(Box::new(move |vars| {
        //let var: &mut IntVar = get_mut_from_handler(vars, &view);
        //var.unsafe_set_value(value);
        //}))
        //})
    }
}
*/
