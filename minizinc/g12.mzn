var 0.0..10.0: x1;
var 0.0..10.0: x2;
var 0.0..10.0: x3;

var 1..9: p;
var 1..9: q;
var 1..9: r;


var float: objective =
    -(100.0 - pow(x1 - 5.0, 2.0) - pow(x2 - 5.0, 2.0) - pow(x3 - 5.0, 2.0)) / 100.0;

constraint pow(x1 - p, 2.0) + pow(x2 - q, 2.0) + pow(x3 - r, 2.0) - 0.0625 <= 0.0;

solve minimize objective;
