array[1..3] of var bool: as;
var bool: r;

constraint r <-> forall(i in 1..3)(as[i]);

solve satisfy;
