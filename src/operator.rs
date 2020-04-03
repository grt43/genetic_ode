//_____________________________________________________________________________
// Author: Garrett Tetrault
// Test ground (for now) of expresion parsing for a genetic program.
//_____________________________________________________________________________

// External imports.
use std::collections::HashMap;
use rand;

const TIME_TOKEN: &'static str = "TIME";
const POS_TOKEN: &'static str = "POS";

//_____________________________________________________________________________
//                                                                Operator Type

#[derive(Copy, Clone)]
#[derive(PartialEq, Eq, Hash)] // Required for use as keys in HashMap.
pub enum Operator {
    Time,
    Position, 
    // We store constants as the bits of a float. Note that most constants 
    // declared will be anonymous. That is, they won't be contained within the
    // operator map, only within expressions.
    Constant(u64),
    Unary(fn(f64) -> f64),
    Binary(fn(f64, f64) -> f64),
}

//_____________________________________________________________________________
//                                                      OperatorMap Type & Impl

pub struct OperatorMap<'a> {
    map: HashMap<Operator, &'a str>,
}

impl<'a> OperatorMap<'a> {
    /* new
    * Create a new instance of OperatorMap that contains time and position with
    * the corresponding tokens declared as constants above.
    * Output:
    *     OperatorMap struct.
    */
    pub fn new() -> OperatorMap<'a> {
        let mut map = HashMap::new();

        // Time and position are required to be in the map.
        // Note that this allows us to assume the map is not empty.
        map.insert(Operator::Time, TIME_TOKEN);
        map.insert(Operator::Position, POS_TOKEN);

        return OperatorMap {map};
    }

    /* len
    * Get the number of operators currently in the operator map.
    * Output:
    *     Size of operator map as usize.
    */
    fn len(&self) -> usize {
        return self.map.len();
    }

    /* insert
    * Insert a given operator and corresponding token into the map.
    * Input:
    *     operator - Instance of operator struct (see above).
    *     token - Name of operator.
    */
    pub fn insert(&mut self, operator: Operator, token: &'a str) {
        // Ensure adherence to token specifications.
        if !token.chars().all(|c: char| c.is_alphanumeric()) {
            panic!("Token {} invalid, \
                cannot contain non-alphanumeric characters.",
                token);
        } else if token.starts_with(|c: char| c.is_numeric()) {
            panic!("Token {} invalid, \
                cannot begin with numeric characters.",
                token);
        } else {
            self.map.insert(operator, token);
        }
    }

    /* get
    * Get the token correpsonding to the given operator from our map.
    * Input:
    *     operator - A reference to an operator.
    * Output:
    *     The token of the operator. 
    */
    pub fn get(&self, operator: &'a Operator) -> Option<&&str> {
        return self.map.get(operator);
    }

    /* rand_operator
    * Get a random operator from our map.
    * Output:
    *     Reference to an operator. 
    */
    pub fn rand_operator(&self) -> &Operator {
        let idx = rand::random::<usize>() % self.len();

        // Note that there are at least two elements in map from new.
        return self.map.keys().skip(idx).next().unwrap();
    }
}