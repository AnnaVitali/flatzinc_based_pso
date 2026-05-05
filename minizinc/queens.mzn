% N-Queens Model
int: n = 16; % Number of queens
array [1..n] of var 1..n: q; % Queen in row i is at column q[i]

include "alldifferent.mzn";

% Constraints
constraint all_different(q); % Each queen in a different column
constraint all_different([q[i] + i | i in 1..n]); % No queen on same upward diagonal
constraint all_different([q[i] - i | i in 1..n]); % No queen on same downward diagonal

solve satisfy;

output ["q = \(q)\n"];