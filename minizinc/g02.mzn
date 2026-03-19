int: n = 20;

array[1..n] of var 0.0..10.0: x;

% ---- objective ----
% numerator   = sum_i cos^4(x_i)  -  2 * prod_i cos^2(x_i)
% denominator = sqrt( sum_i  i * x_i^2 )
% f(x)        = -|numerator / denominator|

var 0.0..20.0:   sum_cos4    = sum(i in 1..n)(pow(cos(x[i]), 4.0));
var 0.0..1.0:    prod_cos2   = product(i in 1..n)(pow(cos(x[i]), 2.0));
var -2.0..20.0:  numerator   = sum_cos4 - 2.0 * prod_cos2;
var 0.0..145.0:  denominator = sqrt(sum(i in 1..n)(i * pow(x[i], 2.0)));
var -20.0..0.0:  objective   = -(abs(numerator) / denominator);

% ---- constraints ----
% g1: prod(x_i) >= 0.75
constraint product(i in 1..n)(x[i]) >= 0.75;

% g2: sum(x_i) <= 7.5 * n = 150
constraint sum(i in 1..n)(x[i]) <= 7.5 * n;

solve minimize objective;


