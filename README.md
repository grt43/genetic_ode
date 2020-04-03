# Genetic ODE
This program is an attempt at implementing a genetic program to find a symbolic formula that best fits the given time and position data.
More specifically, if we are given a vector of times *t* (time_data) and posititons *x* (position_data), we want to find an *f(x, t)* such that the ODE *x' = f(x, t)* fits the given data for some initial conditions.

e.g. If we have some time vector and position vector with values *x[i] = t[i]^2*, then we would want to find the function *f(x, t) = 2t*.

Currently, the user has to edit the time and position vectors directly and can specify the population size and the number of generations.
