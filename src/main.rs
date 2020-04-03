//_____________________________________________________________________________
// Author: Garrett Tetrault
// Entry point.
//_____________________________________________________________________________
// #![allow(dead_code)]

mod operator;
mod expr;
mod population;

use operator::{Operator, OperatorMap};
use expr::{Expr, diff_eq};
use population::Population;

fn main() {
    let mut map = OperatorMap::new();
    map.insert(Operator::Binary(|x, y| x + y), "ADD");
    map.insert(Operator::Binary(|x, y| x - y), "SUB");
    map.insert(Operator::Binary(|x, y| x * y), "MUL");
    map.insert(Operator::Binary(|x, y| x / y), "DIV");

    map.insert(Operator::Unary(|x| x * x), "SQUARE");
    map.insert(Operator::Unary(f64::sqrt), "SQRT");

    // Trigonometric.
    map.insert(Operator::Unary(f64::cos), "COS");
    map.insert(Operator::Unary(f64::sin), "SIN");
    map.insert(Operator::Unary(f64::tan), "TAN");

    map.insert(Operator::Constant(|| 1.0), "ONE");
    map.insert(Operator::Constant(|| 3.14159), "PI");

    let times = vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0];
    let positions = vec![0.0, 1.0, 4.0, 9.0, 16.0, 25.0];

    let size = 300;
    let generations = 5;
    let mut population = Population::new(times, positions);
    population.grow(size, &map);

    for _ in 0..=generations {
        population.population.sort();
        println!("_________________________\n\
        Generation {}:", 
        population.generation);

        for individual in population.population.iter().take(10) {
            println!("{}, fitness = {}", 
                individual.expr.to_string(&map), 
                individual.fitness);
        }
        population.evolve();
    }
}
