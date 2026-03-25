array[1..3] of var bool: as;
var 1..3: b;
var bool: c;

constraint c = as[b];
solve satisfy;
