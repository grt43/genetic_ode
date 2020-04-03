//_____________________________________________________________________________
// Author: Garrett Tetrault
// Test ground (for now) of expresion parsing for a genetic program.
//_____________________________________________________________________________

// External imports.
use rand;
use rand::Rng;

// Internal imports.
use crate::operator::{Operator, ToOperator, OperatorMap};

// Seperating character for printing. Note that we only allow alphanumeric 
// characters for operator tokens.
const SEP_CHAR: char = ' ';

//_____________________________________________________________________________
//                                                             Expr Type & Impl

#[derive(Clone)]
pub struct Expr(Vec<Operator>);

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
                    let c: f64 = rng.gen_range(-100.0, 100.0);
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
        return Expr(operators);
    }

    /* to_string
    */
    pub fn to_string(&self, map: &'a OperatorMap) -> String {
        let mut description = String::from("");
        for operator in self.0.iter() {
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

    /* eval
    * Evaluate the ODE's expression at a given time and position.
    * Input:
    *     time - The value of the time variable.
    *     position - The value of the position variable.
    * Output:
        The value of the evaluated expression.
    */
    pub fn eval(&self, time: f64, position: f64) -> f64 {
        let mut stack: Vec<f64> = Vec::new();

        for operator in self.0.iter().rev() {
            match operator {
                Operator::Time => stack.push(time),
                Operator::Position => stack.push(position),
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

    //_______________________________________________________________
    //                                    Genetic Programming Helpers

    /* sub_expr
    * Get a random valid subexpression of the given expression.
    * Output:
    *     A Expr struct correpsonding to a subexpression.
    */
    pub fn sub_expr(&self) -> Expr {
        let start = rand::random::<usize>() % self.0.len();

        // Find the end point of the subexpression.
        let mut end = start;
        let mut args_needed: i32 = 1;

        for operator in self.0.iter().skip(start) {
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

        let operators = self.0[start..=end].to_vec();

        return Expr(operators);
    }

    /* crossover
    * Replace a random subexression in self with the given sub expression.
    * Output:
    *     A Expr struct correpsonding to the crossover.
    */
    pub fn crossover(&self, sub_expr: &'a Expr) -> Expr {
        let start = rand::random::<usize>() % self.0.len();

        // Find the end point of the subexpression.
        let mut end = start;
        let mut args_needed: i32 = 1;

        for operator in self.0.iter().skip(start) {
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

        // Form symbol vector of new expression.
        let mut operators = self.0[..start].to_vec();
        operators.extend_from_slice(&sub_expr.0);
        operators.extend_from_slice(&self.0[(end+1)..]);

        return Expr(operators);
    }

    pub fn mutate(&mut self) -> Expr {
        let rand = rand::random::<bool>();
        let var = match rand {
            true => Operator::Time,
            false => Operator::Position,
        };
        let expr = Expr(vec![var]);
        return self.crossover(&expr);
    }
}

//_____________________________________________________________________________
//                                                               ODE Simulation
pub mod diff_eq {
    use crate::expr::Expr;

    /* fitness
    * Compute the fitness of an individual against some given data. We asssume 
    * here that an individual will only be tested against the same set of data
    * and as such, we may reuse a fitness value that has been repviously 
    * calculated. 
    */
    pub fn fitness<'a, I>(expr: &'a Expr, 
        times: &mut I, positions: &mut I, step: f64) -> f64 
        where I: Iterator<Item = &'a f64> {

        // Initialize our data bounds.
        let mut prev = (times.next(), positions.next());
        let mut next = (times.next(), positions.next());

        let mut curr_state = (*prev.0.unwrap(), *prev.1.unwrap());

        // Simulate the ODE over the time of the data given.
        let mut fitness = 0.0;

        while next.0 != None {
            // Compute the how well the ODE fits the data. Note that we 
            // test against a linear interpolation between the previous 
            // time and position data and the next time and position 
            // data.
            let prev_state = (*prev.0.unwrap(), *prev.1.unwrap());
            let next_state = (*next.0.unwrap(), *next.1.unwrap());

            // Compute area by the shoelace method.
            let area = (
                (curr_state.0 - next_state.0) *
                (prev_state.1 - curr_state.1) -
                (curr_state.0 - prev_state.0) *
                (next_state.1 - curr_state.1))
                .abs() / 2.0;

            fitness += area;
            
            curr_state = self::next(&expr, curr_state.0, curr_state.1, step);

            // Increment our data bounds.
            if curr_state.0 >= next_state.0 {
                prev = next;
                next = (times.next(), positions.next());
            }
        }

        return fitness;
    }

    /* next
    * Simulate the next step of the ODE using the Runge-Kutta 45 method with 
    * the given initial conditions and time step size.
    */
    fn next<'a>(expr: &'a Expr, 
        time: f64, position: f64, step: f64) -> (f64, f64) {

        // Runge-Kutta 45 method for ODEs.
        let rk45_increment = |dt: f64, dp: f64| 
            expr.eval(time + dt, position + dp);

        let k1 = rk45_increment(0.0, 0.0);
        let k2 = rk45_increment(step / 2.0, step * k1 / 2.0);
        let k3 = rk45_increment(step / 2.0, step * k2 / 2.0);
        let k4 = rk45_increment(step, step * k3);

        let new_time = time + step;
        let new_position = position + (step / 6.0) * 
            (k1 + (2.0 * k2) + (2.0 * k3) + k4);
        
        return (new_time, new_position);
    }
}