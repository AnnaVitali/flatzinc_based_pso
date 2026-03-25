array[1..2] of var bool: pos;
array[1..1] of var bool: neg;

constraint bool_clause(pos, neg);
solve satisfy;
