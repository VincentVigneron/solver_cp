#[macro_use]
extern crate solver_cp;

use solver_cp::branchers::BranchersHandler;
use solver_cp::branchers::brancher::DefaultBrancher;
use solver_cp::branchers::values_selector::MinValueSelector;
use solver_cp::branchers::variables_selector::SequentialVariableSelector;
use solver_cp::constraints::handlers::*;
use solver_cp::search::Solver;
use solver_cp::spaces::Space;
use solver_cp::variables::handlers::*;
use solver_cp::variables::int_var::IntVar;
use solver_cp::variables::int_var::values_int_var::*;

fn main() {
    let mut variables_handler = default_handler::Builder::new();
    let mut constraints_handler = SequentialConstraintsHandler::new();
    let mut branchers_handler = BranchersHandler::new();

    variables!(
        handler = variables_handler;
        let a = var int(3 .. 10);
        let b = var int(2 .. 6);
        let c = var int(1 .. 9);
        );
    constraints!(
        handler = constraints_handler;
        constraint a < b;
        constraint b < c;
    );
    let variables_handler = variables_handler.finalize();

    let variables_selector = SequentialVariableSelector::new(
        vec![a.clone(), b.clone(), c.clone()].into_iter(),
    ).unwrap();
    let values_selector = MinValueSelector::new();
    let brancher = DefaultBrancher::new(variables_selector, values_selector).unwrap();
    branchers_handler.add_specific_brancher(Box::new(brancher));

    let space = Space::new(variables_handler, constraints_handler, branchers_handler);
    let mut solver = Solver::new(space);
    if solver.solve() {
        let solution = solver.solution().unwrap();
        let a = solution.get_variable(&a).value().unwrap();
        let b = solution.get_variable(&b).value().unwrap();
        let c = solution.get_variable(&c).value().unwrap();
        println!("{} < {} < {}", a, b, c);
    } else {
        println!("No solution");
    }
}