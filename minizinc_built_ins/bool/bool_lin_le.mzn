array[1..2] of int: as = [1, 1];
array[1..2] of var bool: bs;
int: c = 1;

constraint as[1]*bool2int(bs[1]) + as[2]*bool2int(bs[2]) <= c;
solve satisfy;
