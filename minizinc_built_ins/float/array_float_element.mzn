array[1..3] of float: as = [1.0, 2.0, 3.0];
var 1..3: b;
var float: c;

constraint c = as[b];
solve satisfy;
