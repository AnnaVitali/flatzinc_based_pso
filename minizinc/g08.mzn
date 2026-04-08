var 0.0..10.0: x1;
var 0.0..10.0: x2;

var -1.0..1.0: numerator = pow(sin(2.0 * 3.141592653589793 * x1), 3.0) * sin(2.0 * 3.141592653589793 * x2);

var 0.0..10000.0: denominator = pow(x1, 3.0) * (x1 + x2);

var float: objective = -(numerator / denominator);


constraint pow(x1, 2.0) - x2 + 1.0 <= 0.0;
constraint 1.0 - x1 + pow(x2 - 4.0, 2.0) <= 0.0;

solve minimize objective;
