var -1.0..1.0: x1;
var -1.0..1.0: x2;

var float: objective = pow(x1, 2.0) + pow(x2 - 1.0, 2.0);

constraint x2 - pow(x1, 2.0) = 0.0;

solve minimize objective;
