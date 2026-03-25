array[1..2] of int: as = [1, -1];
array[1..2] of var int: bs;
int: c = 0;

constraint as[1]*bs[1] + as[2]*bs[2] = c;
solve satisfy;
