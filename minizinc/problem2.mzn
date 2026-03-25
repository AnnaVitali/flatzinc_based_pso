var 13.0..100.0: x1;
var 0.0..100.0: x2;

constraint 100.0 - pow(x1 - 5.0, 2.0) - pow(x2 -5.0, 2.0) <= 0;
constraint pow(x1 - 6.0, 2.0) + pow(x2 - 5.0, 2.0) - 82.81 <= 0;

var float: objective  = pow(x1 - 10.0, 3.0) + pow(x2 - 20.0, 3.0);

solve minimize objective;