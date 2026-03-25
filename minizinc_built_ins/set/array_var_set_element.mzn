array[1..2] of var set of 1..5: as;
var 1..2: i;
var set of 1..5: s;

constraint s = as[i];
solve satisfy;
