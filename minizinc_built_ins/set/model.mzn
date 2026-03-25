int: n = 3; % number of square sizes
enum Sizes = Size(1..n);
array[Sizes] of int: ncopy = [5, 3, 2];
int: maxl = sum(i in Sizes)(i*ncopy[i]);
int: mina = sum(i in Sizes)(i*i*ncopy[i]);

var n..maxl: height;
var n..maxl: width;
var mina .. n*maxl: area = height * width;

enum Squares =  Square(1..sum(ncopy));
array[Squares] of var 0..maxl: x;
array[Squares] of var 0..maxl: y;

% determine size of each square
array[Sizes] of Squares: from = [
  i: if i = min(Sizes) then min(Squares)
     else Square(sum(ncopy[..<i])+1) endif 
  | i in Sizes
];
array[Sizes] of Squares: to = [
  i: if i = max(Sizes) then max(Squares)
     else Square(sum(ncopy[..i])) endif 
  | i in Sizes
];
array[Squares] of var Sizes: size = [
  j: i | i in Sizes, j in from[i]..to[i]
];


constraint area = 35;
constraint height = 7;
constraint width = 5;
constraint x = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
constraint y = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0];