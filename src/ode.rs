//_____________________________________________________________________________
// Author: Garrett Tetrault
// Test ground (for now) of expresion parsing for a genetic program.
//_____________________________________________________________________________

// External imports.
use rand;
use rand::Rng;
use std::ops::RangeInclusive; // Used for sub expressions.

// Internal imports.
use crate::operator::{Operator, ToOperator, OperatorMap};

// Seperating character for printing. Note that we only allow alphanumeric 
// characters for operator tokens.
const SEP_CHAR: char = ' ';

//_____________________________________________________________________________
//                                                                   State Type

#[derive(Copy, Clone, PartialEq)]
pub struct State {
    time: f64,
    position: f64,
}

impl State {
    pub fn new(time: f64, position: f64) -> State {
        return State {time, position};
    }
}

//_____________________________________________________________________________
//                                                             Expr Type & Impl

#[derive(Clone)]
pub struct Expr {
    operators: Vec<Operator>,
}

impl<'a> Expr {
    /* generate
    * Generate a random expression using operators from our given map.
    * Output:
    *     Randomly generated Expr struct.
    */
    pub fn generate(map: &'a OperatorMap) -> Expr {
        // Initialize data scructures to form Expr struct.
        let mut operators = Vec::new();

        // Initialize random number generator.
        let mut rng = rand::thread_rng();

        // Require expression to be not empty.
        let mut args_needed: i32 = 1;
        loop {
            // TODO: rework randomness to be more easily changed.
            // Choose a random class of Operator. Note that we put more weight
            // to choosing the time and position variables even though they 
            // can be found in the map.
            let rand: u32 = rng.gen_range(0, 5);
            match rand {
                0 => { // Time
                    operators.push(Operator::Time);
                    args_needed -= 1;
                },
                1 => { // Position
                    operators.push(Operator::Position);
                    args_needed -= 1;
                },
                2 => { // Anonymous Constant
                    let c: f64 = rng.gen_range(-10.0, 10.0);
                    operators.push(c.to_operator());
                    args_needed -= 1;
                },
                _ => { // Operator
                    let operator = map.rand_operator();
                    operators.push(*operator);

                    // Note that here, there is an argument already required.
                    args_needed += match operator {
                        Operator::Unary(_) => 0,
                        Operator::Binary(_) => 1,
                        _ => -1,
                    }
                },
            }
            // An expression is valid if there are no more arguments needed by 
            // any of the operators.
            if args_needed == 0 {
                break;
            }
        }
        return Expr {operators};
    }

    /* to_string
    */
    pub fn to_string(&self, map: &'a OperatorMap) -> String {
        let mut description = String::from("");
        for operator in self.operators.iter() {
            let token = map.get(operator);
            match token {
                Some(token) => description.push_str(token),
                None => {
                    // Test if it is an anonymous constant.
                    match operator {
                        Operator::Constant(c) => 
                            description.push_str(&f64::from_bits(*c).to_string()),
                        _ => panic!("Encountered operator not in map."),
                    }
                },
            }
            description.push(SEP_CHAR);
        }
        return description;
    }

    //_______________________________________________________________
    //                                     Evaluation and ODE Helpers

    /* eval
    * Evaluate the ODE's expression at a given time and position.
    * Input:
    *     time - The value of the time variable.
    *     position - The value of the position variable.
    * Output:
        The value of the evaluated expression.
    */
    pub fn eval(&self, state: State) -> f64 {
        let mut stack: Vec<f64> = Vec::new();

        for operator in self.operators.iter().rev() {
            match operator {
                Operator::Time => stack.push(state.time),
                Operator::Position => stack.push(state.position),
                Operator::Constant(c) => stack.push(f64::from_bits(*c)),

                // TODO: We are assuming here that the expression is valid.
                //       Need to account for case where it is not.
                Operator::Unary(f) => {
                    let arg = stack.pop().unwrap();
                    stack.push(f(arg));
                }, 
                Operator::Binary(f) => {
                    let arg1 = stack.pop().unwrap();
                    let arg2 = stack.pop().unwrap();
                    stack.push(f(arg1, arg2));
                },
            }
        }

        // If expression is valid, there is exactly one value remaining in the 
        // stack representing the result.
        match stack.len() {
            0 => panic!(
                "Malformed expression, \
                no operands remaining in the stack."),
            1 => return stack.pop().unwrap(),
            _ => panic!(
                "Malformed expression, \
                more than one operand remaining in the stack."),
        }
    }

    /* fitness
    * Compute the fitness of an individual against some given data. We asssume 
    * here that an individual will only be tested against the same set of data
    * and as such, we may reuse a fitness value that has been repviously 
    * calculated. 
    */
    pub fn fitness(&self, states: &'a Vec<State>, step: f64) -> f64 {
        let mut state_iter = states.iter();

        // Initialize our data bounds.
        let mut prev = state_iter.next();
        let mut next = state_iter.next();

        let mut curr_state = State{
            time: prev.unwrap().time, 
            position: prev.unwrap().position,
        };

        // Simulate the ODE over the time of the data given.
        let mut fitness = 0.0;

        while next != None {
            // Compute the how well the ODE fits the data. Note that we 
            // test against a linear interpolation between the previous 
            // time and position data and the next time and position 
            // data.
            let prev_state = prev.unwrap();
            let next_state = next.unwrap();

            // Compute area by the shoelace method.
            let area = (
                (curr_state.time - next_state.time) *
                (prev_state.position - curr_state.position) -
                (curr_state.time - prev_state.time) *
                (next_state.position - curr_state.position))
                .abs() / 2.0;

            fitness += area;
            
            curr_state = self.next(curr_state, step);

            // Increment our data bounds.
            if curr_state.time >= next_state.time {
                prev = next;
                next = state_iter.next();
            }
        }

        return fitness;
    }

    /* simulate
    */
    pub fn simulate(&self, states: Vec<State>, step: f64) {

    }

    /* next
    * Simulate the next step of the ODE using the Runge-Kutta 45 method with 
    * the given initial conditions and time step size.
    */
    fn next(&self, state: State, step: f64) -> State {

        // Runge-Kutta 45 method for ODEs.
        let rk45_increment = |dt: f64, dp: f64| 
            self.eval(State::new(state.time + dt, state.position + dp));

        let k1 = rk45_increment(0.0, 0.0);
        let k2 = rk45_increment(step / 2.0, step * k1 / 2.0);
        let k3 = rk45_increment(step / 2.0, step * k2 / 2.0);
        let k4 = rk45_increment(step, step * k3);

        let new_state = State::new(
            state.time + step,
            state.position + (step / 6.0) * 
                (k1 + (2.0 * k2) + (2.0 * k3) + k4));
        
        return new_state;
    }

    //_______________________________________________________________
    //                                    Genetic Programming Helpers

    /* sub_expr
    * Get a random valid subexpression of the given expression.
    * Output:
    *     A Expr struct correpsonding to a subexpression.
    */
    fn sub_expr(&self) -> RangeInclusive<usize> {
        let start = rand::random::<usize>() % self.operators.len();

        // Find the end point of the subexpression.
        let mut end = start;
        let mut args_needed: i32 = 1;

        for operator in self.operators.iter().skip(start) {
            args_needed += match operator {
                Operator::Unary(_) => 0,
                Operator::Binary(_) => 1,
                _ => -1, 
            };

            // An expression is valid if there are no more arguments needed by 
            // any of the operators.
            if args_needed == 0 {
                break;
            }

            end += 1;
        }

        return start..=end;
    }

    /* crossover
    * Replace a random subexression in self with the given sub expression.
    * Output:
    *     A Expr struct correpsonding to the crossover.
    */
    pub fn crossover(&self, other: &'a Expr) -> Expr {
        let sub_self = self.sub_expr();
        let sub_other = other.sub_expr();

        // Form operator vector of new expression.
        let mut operators = self.operators[..*sub_self.start()].to_vec();
        operators.extend_from_slice(&other.operators[sub_other]);
        operators.extend_from_slice(&self.operators[(*sub_self.end() + 1)..]);

        return Expr {operators};
    }

    /* mutate
    */
    pub fn mutate(&self) -> Expr {
        let rand = rand::random::<bool>();
        let var = match rand {
            true => Operator::Time,
            false => Operator::Position,
        };
        let expr = Expr {operators: vec![var]};
        return self.crossover(&expr);
    }
}

// A struct representing the start and end positions of a sub-expression
// in an expressions' vector of operators.
struct SubExpr {
    start: usize,
    end: usize,
}
