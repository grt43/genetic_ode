//_____________________________________________________________________________
// Author: Garrett Tetrault
// Test ground (for now) of expresion parsing for a genetic program.
//_____________________________________________________________________________
//external imports.
use std::cmp::Ordering;

use rand::Rng;
use rand_distr::Exp;

// Internal imports.
use crate::operator::OperatorMap;
use crate::ode::{State, Expr};

const TIME_STEP: f64 = 0.01;

//_____________________________________________________________________________
//                                                       Individual Type & Impl

#[derive(Clone)]
pub struct Individual {
    pub fitness: f64,
    pub expr: Expr,
}

// Implement an ordering to allow for sorting.
impl Ord for Individual {
    fn cmp(&self, other: &Self) -> Ordering {
        return match (self.fitness.is_nan(), other.fitness.is_nan()) {
            (true, true) => Ordering::Equal,
            (_, true) => Ordering::Less,
            (true, _) => Ordering::Greater,
            (_, _) => self.fitness.partial_cmp(&other.fitness).unwrap(),
        };
    }
}

impl PartialOrd for Individual {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        return Some(self.cmp(other));
    }
}

impl PartialEq for Individual {
    fn eq(&self, other: &Self) -> bool {
        return self.fitness == other.fitness;
    }
}

impl Eq for Individual { }


//_____________________________________________________________________________
//                                                       Population Type & Impl

pub struct Population {
    // Data we are trying to fit.
    states: Vec<State>,

    // Information on the population.
    pub population: Vec<Individual>,
    pub generation: u64,
}

impl<'a> Population {

    /* new
    */
    pub fn new(times: Vec<f64>, positions: Vec<f64>) -> Population {
        if times.len() != positions.len() {
            panic!("Time and position data must be of equal lengths.");
        }
        if times.len() == 0 {
            panic!("Time and position data cannot be emtpy.");
        }

        let states = 
            times.iter().zip(positions.iter())
            .map(|state: (&f64, &f64)| State::new(*state.0, *state.1))
            .collect();

        let population = Vec::new();
        let generation = 0;

        return Population {
            states, 
            population, 
            generation,
        };
    }

    /* grow
    * Grow the population by the specified number of individuals.
    */
    pub fn grow(&mut self, n: usize, map: &'a OperatorMap) {
        for _ in 0..n {
            let expr = Expr::generate(map);
            let fitness = expr.fitness(&self.states, TIME_STEP);
            
            let individual = Individual {fitness, expr};
            self.population.push(individual);
        }
    }

    /* best_fit
    */
    pub fn best_fit(&mut self) -> &Individual {
        self.population.sort();
        let individual = self.population.iter().next().unwrap();
        return &individual;
    }

    /* evolve
    */
    pub fn evolve(&mut self) {
        let size = self.population.len();

        if size == 0 {
            panic!("Cannot evolve population with no individuals.");
        }

        // Note that the population is sorted when we call best_fit.
        let min_fitness = self.best_fit().fitness;

        // Build new population and keep the top 10% unchagned.
        let num_unchanged = size / 10;
        let mut new_population = self.population[0..num_unchanged].to_vec();

        // Initialize random number generator.
        let mut rng = rand::thread_rng();

        // We will use the Pareto distribution due to its heavier tails than 
        // alternatives (like the exponential distribution).
        let lambda = 0.1;
        let exp_distr = Exp::new(lambda).unwrap();
        let mut get_rand = || rng.sample(exp_distr) + min_fitness;

        // Generate the rest of the new population by crossover.
        for _ in 0..(size - num_unchanged) {
            // Get two individuals, randomly chosen proportionally to their 
            // fitness, and crossover.
            let expr1 = &self.closest(get_rand()).expr;
            let expr2 = &self.closest(get_rand()).expr;

            let expr = expr1.crossover(expr2).mutate();

            // Test how well the new expression fits the data.
            let fitness = expr.fitness(&self.states, TIME_STEP);

            let individual = Individual {fitness, expr};
            new_population.push(individual);
        }

        self.population = new_population;
        self.generation += 1;
    }

    /* closest
    * Find the individual with a fitness closest to the given value.
    */
    fn closest(&self, num: f64) -> &Individual {
        let mut iter = self.population.iter();

        let mut prev = iter.next();
        let mut next = iter.next();

        while next != None {
            if prev.unwrap().fitness <= num &&
                next.unwrap().fitness >= num {
                    return prev.unwrap();
                } 

            prev = next;
            next = iter.next();
        }

        // If we don't find a closest individual, we return the first 
        // individiual in our population.
        return prev.unwrap();
    }
}