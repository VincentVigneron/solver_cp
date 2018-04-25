use variables::Array;
use variables::int_var::BoundsIntVar;

constraint_build!(
    struct Propagator = propagator::IncreasingPropagator;
    fn new();
    fn propagate(x: Array<VarType>)
        where VarType: BoundsIntVar<Type=i32>;
    );

pub mod propagator {
    use constraints::{PropagationError, PropagationState, Propagator};
    use variables::Array;
    use variables::int_var::BoundsIntVar;
    #[derive(Debug, Clone)]
    pub struct IncreasingPropagator {}
    impl Propagator for IncreasingPropagator {}
    impl IncreasingPropagator {
        pub fn new() -> IncreasingPropagator {
            IncreasingPropagator {}
        }

        pub fn propagate<VarType: BoundsIntVar<Type = i32>>(
            &self,
            array: &mut Array<VarType>,
        ) -> Result<PropagationState, PropagationError> {
            let len = array.variables.len();
            for i in 0..(len - 1) {
                unsafe {
                    let lhs: &mut VarType = array_get_mut!(array[i]);
                    let rhs: &mut VarType = array_get_mut!(array[i + 1]);
                    let _ = lhs.less_than(rhs);
                }
            }
            for i in 0..(len - 1) {
                unsafe {
                    let lhs: &mut VarType = array_get_mut!(array[len - 2 - i]);
                    let rhs: &mut VarType = array_get_mut!(array[len - 1 - i]);
                    let _ = lhs.less_than(rhs);
                }
            }
            Ok(PropagationState::FixPoint(vec![]))
        }
    }
}
