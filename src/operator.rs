//_____________________________________________________________________________
// Author: Garrett Tetrault
// Test ground (for now) of expresion parsing for a genetic program.
//_____________________________________________________________________________
// External imports.
use std::collections::HashMap;
use rand;

// Interal imports.
use crate::expr::{
    TIME_SYMBOL,
    POSITION_SYMBOL,
    SEP_CHAR,
};

//_____________________________________________________________________________
//                                                                Operator Type
pub enum Operator {
    Unary(fn(f64) -> f64),
    Binary(fn(f64, f64) -> f64),
}

//_____________________________________________________________________________
//                                                      OperatorMap Type & Impl
pub struct OperatorMap<'a>(HashMap<&'a str, Operator>);

impl<'a> OperatorMap<'a> {
	/* new
    * Create a new instance of OperatorMap and ready the operator map.
    * Output:
    *     OperatorMap struct.
    */
    pub fn new() -> OperatorMap<'a> {
        let map = HashMap::new();
        return OperatorMap(map);
    }

    /* len
    * Get the number of operators currently in the operator map.
    * Output:
    *     Size of operator map as usize.
    */
    fn len(&self) -> usize {
        return self.0.len();
    }

    /* insert
    * Insert a given symbol and corresponding operator into the map.
    * Input:
    *     symbol - Name of operator.
    *     operator - Function pointer to operator, either unary or binary.
    */
    pub fn insert(&mut self, symbol: &'a str, operator: Operator) {
        // Ensure adherence to oeprator symbol specs listed above.
        if symbol == TIME_SYMBOL || symbol == POSITION_SYMBOL {
            panic!(
                "Operator symbol \'{}\' \
                conflicts with reserved symbols.", 
                symbol);
        } else if symbol.contains(SEP_CHAR) {
            panic!(
                "Operator symbol \'{}\' \
                cannot contain a \'{}\'.", 
                symbol,
                SEP_CHAR);
        } else if symbol.starts_with(|c: char| c.is_numeric()) {
            panic!(
                "Operator symbol \'{}\' \
                cannot begin with numerical character.", 
                symbol);
        } else { 
            self.0.insert(symbol, operator);
        }
    }

    /* get
    * Get the operator correpsonding to the given symbol from our map.
    * Input:
    *     symbol - Name of operator.
    * Output:
    *     Reference to the desired operator. 
    */
    pub fn get(&self, symbol: &'a str) -> &Operator {
        let operator =  match self.0.get(symbol) {
            Some(op) => op,
            None => {
                panic!(
                    "Operator symbol \'{}\' \
                    is not contained in the map.", 
                    symbol);
            },
        };

        return operator;
    }

    /* get_random
    * Get a random operator from our map.
    * Output:
    *     Reference to an operator. 
    */
    pub fn get_random(&self) -> (&str, &Operator) {
        let idx: usize = rand::random::<usize>() % self.len();
        match self.0.iter().skip(idx).next() {
            Some((name, operator)) => return (name, operator),
            None => {
                panic!(
                    "Empty map, cannot get random element.");
            },
        }
    }
}