var 0.0..1.0: x1;
var 0.0..1.0: x2;
var 0.0..1.0: x3;
var 0.0..1.0: x4;
var 0.0..1.0: x5;
var 0.0..1.0: x6;
var 0.0..1.0: x7;
var 0.0..1.0: x8;
var 0.0..1.0: x9;
var 0.0..100.0: x10;
var 0.0..100.0: x11;
var 0.0..100.0: x12;
var 0.0..1.0: x13;

constraint 2 * x1 + 2 * x2 + x10 + x11 - 10 <= 0;
constraint 2 * x1 + 2 * x3 + x10 + x12 - 10 <= 0;
constraint 2 * x2 + 2 * x3 + x11 + x12 - 10 <= 0;
constraint -8 * x1 + x10 <= 0;
constraint -8 * x2 + x11 <= 0;
constraint -8 * x3 + x12 <= 0;
constraint -2 * x4 - x5 + x10 <= 0;
constraint -2 * x6 - x7 + x11 <= 0;
constraint -2 * x8 - x9 + x12 <= 0;

var float: objective;
constraint objective = 5 * (x1 + x2 + x3 + x4) -5 * (pow(x1, 2) + pow(x2, 2) + pow(x3, 2) + pow(x4, 2)) - (x5 + x6 + x7 + x8 + x9 + x10 + x11 + x12 + x13);

solve minimize objective;