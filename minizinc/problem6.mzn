var 0.0..1.0: x1;
var 0.0..1.0: x2;
var 0.0..1.0: x3;
var 0.0..1.0: x4;
var 0.0..1.0: x5;
var 0.0..20.0: y;

var float: objective = -10.5 * x1 -7.5 * x2 -3.5 * x3 -2.5 * x4 -1.5 * x5 -10.0 * y;

constraint 6 * x1 + 3 * x2 + 2 * x4 + x5 - 6.5 <= 0;
constraint 10 * x1 + 10 * x3 + y <= 20;

solve minimize objective;