var float x2;
var float: x1 = 2 * x2 -1;

constraint pow(x1, 2.0) / 4 + pow(x2, 2.0) - 1 <= 0;

var float: objective  = pow(x1 - 2.0, 2.0) + pow(x2 - 1.0, 2.0);

solve minimize objective;