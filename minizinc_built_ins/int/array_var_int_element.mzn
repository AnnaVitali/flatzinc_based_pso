array[1..3] of var int: as;
var 1..3: b;
var int: c;

constraint c = as[b];
solve satisfy;
