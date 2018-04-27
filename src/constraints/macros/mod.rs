#[macro_export]
macro_rules! constraint_build {
    (@Imports) => {
        use std::marker::PhantomData;
        #[allow(unused_imports)]
        use $crate::variables::{ViewIndex,Variable,VariableState,VariableError};
        use $crate::variables::handlers::{
            VariablesHandler,
            SpecificVariablesHandler,
            get_mut_from_handler};
        use $crate::constraints::{PropagationState};
        use $crate::constraints;
    };
    (
        @Vars struct<$($var_type: ident),+> {
            $( $var: ident: $tvar: ty),+
        }
    ) => {
        struct StructVars<'a, $($var_type: 'a + Variable),+> {
            $($var: &'a mut $tvar),+
        }
    };
    (
        @Views struct {
            $( $var: ident),+
        }
    ) => {
        #[derive(Clone)]
        #[allow(non_camel_case_types)]
        struct StructViews<$($var: Into<ViewIndex> + 'static),+> {
            $($var: $var),+
        }
    };
    (
        @Retrieve struct<$($var_type: ident),+> {
            $( $var: ident: $tvar: ty),+
        }
    ) => {
        #[allow(non_snake_case)]
        #[allow(non_camel_case_types)]
        impl<$($var: Into<ViewIndex>),+> StructViews<$($var),+> {
            #[allow(non_snake_case)]
            #[allow(non_camel_case_types)]
            pub fn retrieve_variables<'a, $($var_type: 'a + Variable),+, H>(
                &self,
                variables_handler: &'a mut H,
                ) -> StructVars<'a, $($var_type),+>
                where H: VariablesHandler $(+SpecificVariablesHandler<$tvar, $var>)+,
                      {
                          unsafe {
                              StructVars {
                                  $(
                                      $var: get_mut_from_handler(
                                          &mut *(variables_handler as *mut _),
                                          &self.$var
                                          )
                                   ),+
                              }
                          }
                      }
        }
    };
    (
        @Constraint struct<$($var_type: ident: $($var_bound: path)|+),+> {
            propagator: $propagator: ty,
            $( $var: ident),+
        }
    ) => {
        #[allow(non_camel_case_types)]
        #[allow(non_snake_case)]
        #[derive(Clone)]
        pub struct Constraint<$($var: Into<ViewIndex> + 'static),+,$($var_type: $($var_bound+)+),+> {
            variables: StructViews<$($var),+>,
            propagator: $propagator,
            $($var_type: PhantomData<$var_type>),+
        }
    };
    (
        @Constraint struct<$($var_type: ident: $($var_bound: path)|+),+> {
            propagator: $propagator: ty,
            state: $state: ty,
            $( $var: ident),+
        }
    ) => {
        #[allow(non_camel_case_types)]
        #[allow(non_snake_case)]
        #[derive(Clone)]
        pub struct Constraint<$($var: Into<ViewIndex> + 'static),+,$($var_type: $($var_bound+)+),+> {
            variables: StructViews<$($var),+>,
            propagator: $propagator,
            state: Option<$state>,
            $($var_type: PhantomData<$var_type>),+
        }
    };
    (
        @Propagate struct<$($var_type: ident: $($var_bound: path)|+),+> {
            $( $var: ident: $tvar: ty),+
        }
        propagate: $fnpropagate: ident;
    ) => {
        #[allow(non_camel_case_types)]
        #[allow(non_snake_case)]
        impl<$($var: 'static + Clone + Into<ViewIndex>),+,
        $($var_type: 'static + Variable + $($var_bound+)+),+,
        H: 'static + Clone + VariablesHandler $(+SpecificVariablesHandler<$tvar, $var>)+
            > constraints::Constraint<H>
            for Constraint<$($var),+,$($var_type),+> {

            fn propagate(&mut self, variables_handler: &mut H)
                -> Result<PropagationState, VariableError> {
                let variables =
                    self.variables.retrieve_variables(variables_handler);
                self.propagator.$fnpropagate::<$($var_type),+>(
                    $(variables.$var),+)
            }

            fn box_clone(&self) -> Box<constraints::Constraint<H>> {
                let ref_self: &Constraint<$($var),+, $($var_type),+> = &self;
                let cloned: Constraint<$($var),+, $($var_type),+> =
                    <Constraint<$($var),+,$($var_type),+> as Clone>::clone(ref_self);

                Box::new(cloned) as Box<constraints::Constraint<H>>
            }

            fn retrieve_changed_views(
                &self,
                variables_handler: &mut H
            ) -> Box<Iterator<Item = (ViewIndex, VariableState)>> {
                use std::iter;
                let states = vec![
                    $(
                        variables_handler.retrieve_states(
                            iter::once(&self.variables.$var))
                    ),+
                ];
                let changed: Vec<_> = states.into_iter()
                    .flat_map(|states| states)
                    .filter(|&(_,ref state)| *state == VariableState::NoChange)
                    .collect();
                Box::new(changed.into_iter())
            }

            fn affected_by_changes<'a>(
                &self,
                states: &mut Iterator<Item = &'a (ViewIndex, VariableState)>,
            ) -> bool {
                unimplemented!()
            }

            fn affected_by_change(&self, view_index: &ViewIndex, state: &VariableState) -> bool {
                unimplemented!()
            }
        }
    };
    (
        @Propagate struct<$($var_type: ident: $($var_bound: path)|+),+> {
            $( $var: ident: $tvar: ty),+
        }
        propagate: $fnpropagate: ident;
        struct State = $state: ty;
    ) => {
        #[allow(non_camel_case_types)]
        #[allow(non_snake_case)]
        impl<$($var: 'static + Clone + Into<ViewIndex>),+,
        $($var_type: 'static + $($var_bound+)+),+,
        H: 'static + Clone + VariablesHandler $(+SpecificVariablesHandler<$tvar, $var>)+
            > constraints::Constraint<H>
            for Constraint<$($var),+,$($var_type),+> {

            fn propagate(&mut self, variables_handler: &mut H)
                -> Result<PropagationState, VariableError>
                {
                let variables =
                    self.variables.retrieve_variables(variables_handler);
                self.propagator.$fnpropagate::<$($var_type),+>(
                    $(variables.$var),+,
                    &mut self.state)
                }

            fn box_clone(&self) -> Box<constraints::Constraint<H>> {
                let ref_self: &Constraint<$($var),+, $($var_type),+> = &self;
                let cloned: Constraint<$($var),+, $($var_type),+> =
                    <Constraint<$($var),+,$($var_type),+> as Clone>::clone(ref_self);

                Box::new(cloned) as Box<constraints::Constraint<H>>
            }
            fn retrieve_changed_views(
                &self,
                variables_handler: &mut H
            ) -> Box<Iterator<Item = (ViewIndex, VariableState)>> {
                use std::iter;
                let states = vec![
                    $(
                        variables_handler.retrieve_states(
                            iter::once(&self.variables.$var))
                    ),+
                ];
                let changed: Vec<_> = states.into_iter()
                    .flat_map(|states| states)
                    .filter(|&(_,ref state)| *state == VariableState::NoChange)
                    .collect();
                Box::new(changed.into_iter())
            }

            fn affected_by_changes<'a>(
                &self,
                states: &mut Iterator<Item = &'a (ViewIndex, VariableState)>,
            ) -> bool {
                unimplemented!()
            }

            fn affected_by_change(&self, view_index: &ViewIndex, state: &VariableState) -> bool {
                unimplemented!()
            }
        }
    };
    (
        @ConstraintImpl struct<$($var_type: ident: $($var_bound: path)|+),+> {
            propagator: $propagator: ty,
            $( $var: ident),+
        }
        new: $fnnew: ident($( $param: ident: $tparam: ty),*);
    ) => {
        #[allow(non_camel_case_types)]
        #[allow(non_snake_case)]
        impl<$($var),+, $($var_type),+> Constraint<$($var),+, $($var_type),+>
            where
                $($var: Into<ViewIndex> + Clone),+,
                $($var_type: $($var_bound+)+),+
                {

                    #[allow(non_camel_case_types)]
                    #[allow(non_snake_case)]
                    pub fn $fnnew($($var: &$var),+,$($param: $tparam),*)
                        -> Constraint<$($var),+, $($var_type),+>
                        {
                            use $crate::variables::AllDisjoint;
                            let valid = vec![$($var.clone().into()),+]
                                .into_iter()
                                .all_disjoint();
                            if let Err((left,right)) = valid {
                                panic!("All views must refer to different variables for: \"{}\". Variable {:?} and {:?} are tied together",
                                       stringify!($propagator),
                                       left,
                                       right
                                       );
                            }

                            Constraint {
                                propagator: <$propagator>::$fnnew($($param),*),
                                variables: StructViews {
                                    $($var: $var.clone()),+,
                                },
                                $($var_type: PhantomData),+
                            }
                        }
                }
    };
    (
        @ConstraintImpl struct<$($var_type: ident: $($var_bound: path)|+),+> {
            propagator: $propagator: ty,
            state: $state: ty,
            $( $var: ident),+
        }
        new: $fnnew: ident($( $param: ident: $tparam: ty),*);
    ) => {
        #[allow(non_camel_case_types)]
        #[allow(non_snake_case)]
        impl<$($var),+, $($var_type),+> Constraint<$($var),+, $($var_type),+>
            where
                $($var: Into<ViewIndex> + Clone),+,
                $($var_type: $($var_bound+)+),+
                {

                    #[allow(non_camel_case_types)]
                    #[allow(non_snake_case)]
                    pub fn $fnnew($($var: &$var),+,$($param: $tparam),*)
                        -> Constraint<$($var),+, $($var_type),+>
                        {
                            use $crate::variables::AllDisjoint;
                            let valid = vec![$($var.clone().into()),+]
                                .into_iter()
                                .all_disjoint();
                            if let Err((left,right)) = valid {
                                panic!("All views must refer to different variables for: \"{}\". Variable {:?} and {:?} are tied together",
                                       stringify!($propagator),
                                       left,
                                       right
                                       );
                            }

                            Constraint {
                                propagator: <$propagator>::$fnnew($($param),*),
                                state: None,
                                variables: StructViews {
                                    $($var: $var.clone()),+,
                                },
                                $($var_type: PhantomData),+
                            }
                        }
                }
    };
    (
        @New struct<$($var_type: ident: $($var_bound: path)|+),+> {
            $( $var: ident),+
        }
        new: $fnnew: ident($( $param: ident: $tparam: ty),*);
    ) => {
        #[allow(non_camel_case_types)]
        #[allow(non_snake_case)]
        pub fn $fnnew<$($var),+,$($var_type),+>($($var: &$var),+,$($param: $tparam),*)
            -> Constraint<$($var),+,$($var_type),+>
            where
            $($var: Into<ViewIndex> + Clone),+,
            $($var_type: $($var_bound+)+),+
            {
                Constraint::$fnnew($($var),+,$($param),*)
            }
    };
    (
        $(#[$outer:meta])*
        struct Propagator = $propagator: ty;
        fn $fnnew: ident($( $param: ident: $tparam: ty),*);
        fn $fnpropagate: ident($( $var: ident: $tvar: ty),+)
        where  $($var_type: ident: $($var_bound: path)|+),+;
    ) => {
        constraint_build!(@Imports);

        constraint_build!(
            @Vars struct<$($var_type),+> {
                $($var: $tvar),+
            });
        constraint_build!(
            @Views struct {
                $($var),+
            });
        constraint_build!(
            @Retrieve struct<$($var_type),+> {
                $($var: $tvar),+
            });
        constraint_build!(
            @Constraint struct<$($var_type: $($var_bound)|+),+> {
                propagator: $propagator,
                $($var),+
            });
        constraint_build!(
            @Propagate struct<$($var_type: $($var_bound)|+),+> {
                $($var: $tvar),+
            }
            propagate: $fnpropagate;
            );
        constraint_build!(
            @ConstraintImpl struct<$($var_type: $($var_bound)|+),+> {
                propagator: $propagator,
                $($var),+
            }
            new: $fnnew($($param: $tparam),*);
            );
        constraint_build!(
            @New struct<$($var_type: $($var_bound)|+),+> {
                $($var),+
            }
            new: $fnnew($($param: $tparam),*);
            );
    };
    (
        $(#[$outer:meta])*
        struct Propagator = $propagator: ty;
        fn $fnnew: ident($( $param: ident: $tparam: ty),*);
        fn $fnpropagate: ident($( $var: ident: $tvar: ty),+) -> Option<$state: ty>
        where  $($var_type: ident: $($var_bound: path)|+),+;
    ) => {
        constraint_build!(@Imports);

        constraint_build!(
            @Vars struct<$($var_type),+> {
                $($var: $tvar),+
            });
        constraint_build!(
            @Views struct {
                $($var),+
            });
        constraint_build!(
            @Retrieve struct<$($var_type),+> {
                $($var: $tvar),+
            });
        constraint_build!(
            @Constraint struct<$($var_type: $($var_bound)|+),+> {
                propagator: $propagator,
                state: $state,
                $($var),+
            });
        constraint_build!(
            @Propagate struct<$($var_type: $($var_bound)|+),+> {
                $($var: $tvar),+
            }
            propagate: $fnpropagate;
            struct State = $state;
            );
        constraint_build!(
            @ConstraintImpl struct<$($var_type: $($var_bound)|+),+> {
                propagator: $propagator,
                state: $state,
                $($var),+
            }
            new: $fnnew($($param: $tparam),*);
            );
        constraint_build!(
            @New struct<$($var_type: $($var_bound)|+),+> {
                $($var),+
            }
            new: $fnnew($($param: $tparam),*);
            );
    };
}

#[macro_export]
macro_rules! constraints {
    () => {};
    (handler = $handler: ident;) => {};
    (handler = $handler: ident; constraint increasing($x:ident); $($tail:tt)*) => {
        {
            $handler.add(Box::new($crate::constraints::increasing::new(&$x)));
            constraints!(handler = $handler; $($tail)*);
        }
    };
    (handler = $handler: ident; constraint $x:ident < $y: ident; $($tail:tt)*) => {
        {
            $handler.add(Box::new(
                    $crate::constraints::arithmetic::less_than::new(&$x, &$y)));
            constraints!(handler = $handler; $($tail)*);
        }
    };
    (handler = $handler: ident; constraint $x:ident <= $y: ident; $($tail:tt)*) => {
        {
            $handler.add(Box::new(
                    $crate::constraints::arithmetic::less_or_equal_than::new(&$x, &$y)));
            constraints!(handler = $handler; $($tail)*);
        }
    };
    (handler = $handler: ident; constraint $x:ident > $y: ident; $($tail:tt)*) => {
        {
            $handler.add(Box::new(
                    $crate::constraints::arithmetic::greater_than::new(&$x, &$y)));
            constraints!(handler = $handler; $($tail)*);
        }
    };
    (handler = $handler: ident; constraint $x:ident >= $y: ident; $($tail:tt)*) => {
        {
            $handler.add(Box::new(
                    $crate::constraints::arithmetic::greater_or_equal_than::new(&$x, &$y)));
            constraints!(handler = $handler; $($tail)*);
        }
    };
    (handler = $handler: ident; constraint $x:ident == $y: ident; $($tail:tt)*) => {
        {
            $handler.add(Box::new(
                    $crate::constraints::arithmetic::equal::new(&$x, &$y)));
            constraints!(handler = $handler; $($tail)*);
        }
    };
    (handler = $handler: ident; constraint $x:ident |==| $y: ident; $($tail:tt)*) => {
        {
            $handler.add(Box::new(
                    $crate::constraints::arithmetic::equal_on_bounds::new(&$x, &$y)));
            constraints!(handler = $handler; $($tail)*);
        }
    };
}
