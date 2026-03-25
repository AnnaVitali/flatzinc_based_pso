array[1..2] of set of 1..5: as = [{1,2}, {3}];
var 1..2: i;
var set of 1..5: s;

constraint s = as[i];
solve satisfy;
