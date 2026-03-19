var 100.0..10000.0: x1;
var 1000.0..10000.0: x2;
var 1000.0..10000.0: x3;
var 10.0..1000.0: x4;
var 10.0..1000.0: x5;
var 10.0..1000.0: x6;
var 10.0..1000.0: x7;
var 10.0..1000.0: x8;

var float: objective = x1 + x2 + x3;

constraint -1.0 + 0.0025 * (x4 + x6) <= 0.0;
constraint -1.0 + 0.0025 * (x5 + x7 - x4) <= 0.0;
constraint -1.0 + 0.01 * (x8 - x5) <= 0.0;
constraint -x1 * x6 + 833.33252 * x4 + 100.0 * x1 - 83333.333 <= 0.0;
constraint -x2 * x7 + 1250.0 * x5 + x2 * x4 - 1250.0 * x4 <= 0.0;
constraint -x3 * x8 + 1250000.0 + x3 * x5 - 2500.0 * x5 <= 0.0;

solve minimize objective;
