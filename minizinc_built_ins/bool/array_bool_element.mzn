array[1..3] of bool: as = [true, false, true];
var 1..3: b;
var bool: c;

constraint c = as[b];
solve satisfy;
