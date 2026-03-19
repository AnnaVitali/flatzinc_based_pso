var 0.0..1200.0: x1;
var 0.0..1200.0: x2;
var -0.55..0.55: x3;
var -0.55..0.55: x4;

var float: objective =
	3.0 * x1
	+ 0.000001 * pow(x1, 3.0)
	+ 2.0 * x2
	+ (0.000002 / 3.0) * pow(x2, 3.0);

constraint -x4 + x3 - 0.55 <= 0.0;
constraint -x3 + x4 - 0.55 <= 0.0;

constraint 1000.0 * sin(-x3 - 0.25) + 1000.0 * sin(-x4 - 0.25) + 894.8 - x1 = 0.0;
constraint 1000.0 * sin(x3 - 0.25) + 1000.0 * sin(x3 - x4 - 0.25) + 894.8 - x2 = 0.0;
constraint 1000.0 * sin(x4 - 0.25) + 1000.0 * sin(x4 - x3 - 0.25) + 1294.8 = 0.0;

solve minimize objective;
