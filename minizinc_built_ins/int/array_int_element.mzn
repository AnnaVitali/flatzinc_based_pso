array[1..3] of int: as = [10, 20, 30];

var 1..3: b;
var int: c;

constraint c = as[b];

solve satisfy;
