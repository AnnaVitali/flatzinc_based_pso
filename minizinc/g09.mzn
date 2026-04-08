var -10.0..10.0: x1;
var -10.0..10.0: x2;
var -10.0..10.0: x3;
var -10.0..10.0: x4;
var -10.0..10.0: x5;
var -10.0..10.0: x6;
var -10.0..10.0: x7;

var float: objective = pow(x1 - 10.0, 2.0) + 5.0 * pow(x2 - 12.0, 2.0) + pow(x3, 4.0) + 3.0 * pow(x4 - 11.0, 2.0) + 10.0 * pow(x5, 6.0) + 7.0 * pow(x6, 2.0)+ pow(x7, 4.0)
    - 4.0 * x6 * x7 - 10.0 * x6 - 8.0 * x7;

constraint -127.0 + 2.0 * pow(x1, 2.0) + 3.0 * pow(x2, 4.0) + x3 + 4.0 * pow(x4, 2.0) + 5.0 * x5 <= 0.0;
constraint -282.0 + 7.0 * x1 + 3.0 * x2 + 10.0 * pow(x3, 2.0) + x4 - x5 <= 0.0;
constraint -196.0 + 23.0 * x1 + pow(x2, 2.0) + 6.0 * pow(x6, 2.0) - 8.0 * x7 <= 0.0;
constraint 4.0 * pow(x1, 2.0) + pow(x2, 2.0) - 3.0 * x1 * x2 + 2.0 * pow(x3, 2.0) + 5.0 * x6 - 11.0 * x7 <= 0.0;

solve minimize objective;
