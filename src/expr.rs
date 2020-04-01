//_____________________________________________________________________________
// Author: Garrett Tetrault
// Test ground (for now) of expresion parsing for a genetic program.
//_____________________________________________________________________________
// External imports.
use rand::Rng;

// Internal imports.
use crate::operator::{
    Operator,
    OperatorMap,
};

//_____________________________________________________________________________
//                                        Expression Syntax & Symbol Definition
/*
    Expressions are stored with two components: a string description and a 
    vector of symbols.
    Symbols consist of three main components:
    - Variables, in time and position, denoted by "TIME" and "POS" 
      respectively.
    - Constants, denoted "<f64>" where <f64> is the string representation of an
      f64 number. 
    - Operators, either unary of binary, denoted by a string. Note that 
      operator symbols beginning with a numeral or containing a " " are NOT 
      allowed.
    We use Polish notation for our text description and similarly store the 
    symbols in a vector.
    e.g.

    [MUL, ADD, COS, TIME, POS, 3.14159] = (cos(t) + x) * 3.14159
*/

pub const TIME_SYMBOL: &'static str = "TIME";
pub const POSITION_SYMBOL: &'static str = "POS";
pub const SEP_CHAR: char = ' ';

pub const TIME_DELTA: f64 = 0.01;


//_____________________________________________________________________________
//                                                           Symbol Type & Impl
#[derive(Copy, Clone)]
enum Symbol<'a> {
    Time,
    Position,
    Constant(f64),
    Operator(&'a Operator),
}

//_____________________________________________________________________________
//                                                             Expr Type & Impl
#[derive(Clone)]
pub struct Expr<'a> {
    description: String,
    symbols: Vec<Symbol<'a>>,
}

impl<'a> Expr<'a> {
    /* len
    * Get the number of operators and operands in the expression.
    */
    pub fn len(&self) -> usize {
        return self.symbols.len();
    }

    /* description
    * Get the description of a expression. Note that descriptions are given in
    * Polish notation.
    */
    pub fn description(&self) -> &str {
        return &self.description;
    }

    //_______________________________________________________________
    //                                         Evaluation and Fitness

    /* eval
    * Evaluate an expression at a given time and position.
    * Input:
    *     time - The value of the time variable.
    *     position - The value of the position variable.
    * Output:
        The value of the evaluated expression.
    */
    pub fn eval(&self, time: f64, position: f64) -> f64 {
        let mut stack: Vec<f64> = Vec::new();

        for symbol in self.symbols.iter().rev() {
            match symbol {
                Symbol::Time => stack.push(time),
                Symbol::Position => stack.push(position),

                Symbol::Constant(c) => stack.push(*c),

                // TODO account for malformed expressions.
                Symbol::Operator(operator) => {
                    match operator {
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
    pub fn fitness(&self, time_data: &Vec<f64>, position_data: &Vec<f64>) -> f64 {
        // Allow forsome assumptions on the data.
        if time_data.len() != position_data.len() {
            panic!(
                "Time and position data \
                must be of equal lengths.");
        }
        if time_data.len() == 0 {
            panic!(
                "Time and position data \
                cannot be emtpy.");
        }

        let mut time_iter = time_data.iter();
        let mut position_iter = position_data.iter();

        // Initialize our data bounds.
        let mut prev_time = time_iter.next();
        let mut next_time = time_iter.next();

        let mut prev_pos = position_iter.next(); 
        let mut next_pos = position_iter.next();

        // Note that we are free to unwrap as data is asserted to be 
        // non-empty.
        let mut cur_time = *prev_time.unwrap();
        let mut cur_pos = *prev_pos.unwrap();

        // Simulate the ODE over the time of the data given.
        let mut fitness = 0.0;
        while next_time != None {
            // Compute the how well the ODE fits the data. Note that we 
            // test against a linear interpolation between the previous 
            // time and position data and the next time and position 
            // data.

            // Distance from previous to current.
            let a = f64::sqrt(
                (cur_time - prev_time.unwrap()).powi(2) + 
                (cur_pos - prev_pos.unwrap()).powi(2));

            // Distance from previous to next.
            let b = f64::sqrt(
                (next_time.unwrap() - prev_time.unwrap()).powi(2) + 
                (next_pos.unwrap() - prev_pos.unwrap()).powi(2));

            // Distance from current to next.
            let c = f64::sqrt(
                (next_time.unwrap() - cur_time).powi(2) + 
                (next_pos.unwrap() - cur_pos).powi(2));
            
            // See Heron's Formula for details on the calculations.
            let s = (a + b + c) / 2.0;
            // TODO: Find roundoff error causing area to be negative.
            let h = (2.0 / b) * f64::sqrt(f64::abs(s * (s - a) * (s - b) * (s - c)));
            fitness += h;
            
            // Runge-Kutta 45 method for ODEs.
            let k1 = TIME_DELTA * self.eval(cur_time, cur_pos);
            let k2 = TIME_DELTA * self.eval(cur_time + TIME_DELTA / 2.0, cur_pos + k1 / 2.0);
            let k3 = TIME_DELTA * self.eval(cur_time + TIME_DELTA / 2.0, cur_pos + k2 / 2.0);
            let k4 = TIME_DELTA * self.eval(cur_time + TIME_DELTA, cur_pos + k3);

            cur_time += TIME_DELTA;
            cur_pos += (k1 + 2.0*k2 + 2.0*k3 + k4) / 6.0;

            // Increment our data bounds.
            if cur_time >= *next_time.unwrap() {
                prev_time = next_time;
                next_time = time_iter.next();

                prev_pos = next_pos;
                next_pos = position_iter.next();
            }
        }

        return fitness;
    }

    //_______________________________________________________________
    //                                    Genetic Programming Helpers

    /* sub_expr
    * Get a random valid subexpression of the given expression.
    * Output:
    *     A Expr struct correpsonding to a subexpression.
    */
    pub fn sub_expr(&self) -> Expr<'a> {
        let start: usize = rand::random::<usize>() % self.len();

        // Find the end point of the subexpression.
        let mut end = start;
        let mut args_needed: u32 = 1;
        for symbol in self.symbols.iter().skip(start) {
            args_needed = match symbol {
                Symbol::Operator(operator) => {
                    match operator {
                        Operator::Unary(_) => args_needed,
                        Operator::Binary(_) => args_needed + 1,
                    }
                },
                _ => args_needed - 1,
            };

            // An expression is valid if there are no more arguments needed by 
            // any of the operators.
            if args_needed == 0 {
                break;
            }

            end += 1;
        }

        // Find corresponding start and end points in description.
        let description: String = 
            self.description
            .split(SEP_CHAR)
            .map(|s: &str| format!("{}{}", s, SEP_CHAR)) // Add seperations. 
            .skip(start) // Ignore all items before the start.
            .take(end - start + 1) // Take only the items between start and end.
            .collect::<String>() // Convert back to a single vector.
            .trim_matches(SEP_CHAR) // Trim leading and tailing whitespace.
            .to_string(); // Convert back to String.

        let symbols = self.symbols[start..=end].to_vec();

        return Expr {description, symbols};
    }

    /* crossover
    * Replace a random subexression in self with the given sub expression.
    * Output:
    *     A Expr struct correpsonding to the crossover.
    */
    pub fn crossover(&self, sub_expr: Expr<'a>) -> Expr<'a> {
        let start: usize = rand::random::<usize>() % self.len();

        // Find the end point of the subexpression.
        let mut end = start;
        let mut args_needed: u32 = 1;
        for symbol in self.symbols.iter().skip(start) {
            args_needed = match symbol {
                Symbol::Operator(operator) => {
                    match operator {
                        Operator::Unary(_) => args_needed,
                        Operator::Binary(_) => args_needed + 1,
                    }
                },
                _ => args_needed - 1,
            };

            // An expression is valid if there are no more arguments needed by 
            // any of the operators.
            if args_needed == 0 {
                break;
            }

            end = end + 1;
        }

        // TODO: Simplify this logic.
        // Form description of new expression.
        let mut description: String = 
            self.description
            .split(SEP_CHAR)
            .map(|s: &str| format!("{}{}", s, SEP_CHAR)) // Add seperations. 
            .take(start) // Take items before the subexpression.
            .collect();
        description.push_str(sub_expr.description());

        description.push(SEP_CHAR);

        description.push_str(
            &self.description
            .split(SEP_CHAR)
            .map(|s: &str| format!("{}{}", s, SEP_CHAR)) // Add seperations. 
            .skip(end+1) // Skip to after subexpression.
            .collect::<String>());

        // Form symbol vector of new expression.
        let mut symbols: Vec<Symbol> = self.symbols[..start].to_vec();
        symbols.extend(&sub_expr.symbols);
        symbols.extend(&self.symbols[(end+1)..]);

        return Expr{description, symbols};
    }
}

//_____________________________________________________________________________
//                                                           Parser Type & Impl

pub struct ExprParser<'a> {
    map: OperatorMap<'a>,
}

impl<'a> ExprParser<'a> {
    /* new
    * Create a new instance of an ExprParser and ready the operator map.
    * Output:
    *     Empty ExprParser struct.
    */
    pub fn new() -> ExprParser<'a> {
        let map = OperatorMap::new();
        return ExprParser {map};
    }

    pub fn insert(&mut self, symbol: &'a str, operator: Operator) {
        self.map.insert(symbol, operator);
    }

    /* from_description
    * From a string description (see expression syntax above), create an 
    * expression. Program panics if description contains operators not in our 
    * map or if operator names are invalid. Note that we do NOT check if the 
    * expression is valid here (e.g. "ADD TIME" is invalid as "ADD" takes two 
    * arguments). 
    * Input:
    *     description - Description of expression.
    * Output:
    *     Expr struct corresponding to description.
    */
    pub fn from_description(&self, description: &'a str) -> Expr {
        let mut symbols: Vec<Symbol> = Vec::new();

        for name in description.split(" ") {
            match name {
                // Check against reserved names.
                TIME_SYMBOL => symbols.push(Symbol::Time),
                POSITION_SYMBOL => symbols.push(Symbol::Position),

                // Check for numeric constant value.
                _ if name.starts_with(|c: char| c.is_numeric()) => {
                    let constant = name.parse::<f64>();
                    match constant {
                        Err(_) => {
                            panic!(
                                "Constant {} \
                                is not a valid numeric value.", 
                                name);
                        },
                        Ok(c) => symbols.push(Symbol::Constant(c)),
                    }
                },

                // Otherwise, we are working with an operator name.
                _ => {
                    let operator: &Operator = self.map.get(name);
                    symbols.push(Symbol::Operator(operator));
                },
            }
        }
        return Expr{description: description.to_string(), symbols};
    }

    //_______________________________________________________________
    //                                 Genetic Programming Algorithms

    /* generate
    * Generate a random expression using operators from our given map.
    * Output:
    *     Randomly generated Expr struct.
    */
    pub fn generate(&self) -> Expr {
        // Initialize data scructures to form Expr struct.
        let mut description: String = String::from("");
        let mut symbols: Vec<Symbol> = Vec::new();

        // Initialize random number generator.
        let mut rng = rand::thread_rng();

        // Require expression to be not empty.
        let mut args_needed: u32 = 1;
        loop {
            // TODO: rework randomness to be more easily changed.
            // Choose a random class of Symbol.
            let rand: u32 = rng.gen_range(0, 5);
            match rand {
                0 => { // Time
                    description.push_str(TIME_SYMBOL);
                    symbols.push(Symbol::Time);

                    args_needed -= 1;
                },
                1 => { // Position
                    description.push_str(POSITION_SYMBOL);
                    symbols.push(Symbol::Position);

                    args_needed -= 1;
                },
                2 => { // Constant
                    // TODO: Allow for easy change of bounds.
                    let constant: f64 = rng.gen_range(-100.0, 100.0);

                    description.push_str(&constant.to_string());
                    symbols.push(Symbol::Constant(constant));

                    args_needed -= 1;
                },
                _ => { // Operator
                    let (name, operator) = self.map.get_random();
                    
                    description.push_str(name);
                    symbols.push(Symbol::Operator(operator));

                    // Note that here, there is an argument already required.
                    args_needed += match operator {
                        Operator::Unary(_) => 0,
                        Operator::Binary(_) => 1,
                    }
                },
            }
            description.push(SEP_CHAR);

            // An expression is valid if there are no more arguments needed by 
            // any of the operators.
            if args_needed == 0 {
                break;
            }
        }

        let expr = Expr{description, symbols};
        return expr;
    }
}