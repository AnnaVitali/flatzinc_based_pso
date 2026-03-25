array[1..3] of var float: as;
var 1..3: b;
var float: c;

constraint c = as[b];
solve satisfy;
