//_____________________________________________________________________________
// Author: Garrett Tetrault
// Entry point.
//_____________________________________________________________________________
// #![allow(dead_code)]

mod operator;
mod expr;
mod population;

use operator::OperatorMap;
use population::Population;

fn main() {
    let mut map = OperatorMap::new();

    // Basic arithmetic operators.
    map.insert((|x, y| x + y) as fn(f64, f64) -> f64, "ADD");
    map.insert((|x, y| x - y) as fn(f64, f64) -> f64, "SUB");
    map.insert((|x, y| x * y) as fn(f64, f64) -> f64, "MUL");
    map.insert((|x, y| x / y) as fn(f64, f64) -> f64, "DIV");

    map.insert((|x| x * x) as fn(f64) -> f64, "SQUARE");
    map.insert(f64::sqrt as fn(f64) -> f64, "SQRT");

    // Trigonometric functions.
    // map.insert(f64::cos as fn(f64) -> f64, "COS");
    // map.insert(f64::sin as fn(f64) -> f64, "SIN");
    // map.insert(f64::tan as fn(f64) -> f64, "TAN");

    // Logarithmic functions.
    map.insert(f64::exp as fn(f64) -> f64, "EXP");
    map.insert(f64::ln as fn(f64) -> f64, "LN");

    // We can use named constants too.
    map.insert(3.0, "THREE");
    map.insert(3.14159, "PI");

    // Specify data.
    let times = vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0];
    let positions = vec![0.0, 3.0, 6.0, 9.0, 12.0, 15.0];

    // Construct population and simulate.
    let size = 300;
    let generations = 10;
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
