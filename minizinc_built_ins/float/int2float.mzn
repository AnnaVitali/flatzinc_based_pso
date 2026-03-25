var int: x;
var float: y;

constraint y = x;  % implicit int → float cast

solve satisfy;
