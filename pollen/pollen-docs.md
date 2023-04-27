===== Pollen Features =====

1.1 Basics

Variable initialization:

```
t v1;
v1 = e1;
```
or 

```
type v2 = e2;
```

For now, to keep things simple, variables can only be declared once within the
same scope. This means that
```
int v = 1;
bool v = true;
```
is semantically invalid. 

2. Types
2.1 Basic Types

`T = int | bool | char`


2.2 Abstract Data Types

Array types are expressed as `T[]`, and can be nested up to a depth of 4.
(We could extend the functionality to allow for deeper nesting, but for now
this should be more than fine.)

Strings are essentially lists of characters.

Record types: `R = {f1: t1, f2 : t2, ..., fn : tn}` if `t1 ... t2 in t`

Without considering pangenomic types, our type system would look something
like:

`t = T | T[] | String`


2.3 Pangenomic types

`Segment | Step | Path | Edge | Handle | base | Strand`

Where `Segment`, `Step`, `Path`, `Edge`, and `Hande` are all named record types.

So,

`T = int | bool | char | base`

`P = Segment | Step | Path | Edge | Handle | Strand`

`t = T | T[] | String | P | List<T> | List<P> | R`

The `Set` type is not exposed to the user, nor is the keyword available.


3. If/else

If statement:
```
if b {
    ...
}
```

If/else statement:
```
if b {
    ...
} else {
    ...
}
```

If/elif:
```
if b1 {
    ...
} elif b2 {
    ...
} [else {...}] // Optional catch-all else block
```

4. Loops
4.1 While Loops

```
while i < len {
    ...
}
```

This is in a similar style to the for-each loop syntax given below, which I find
more intuitive than the Java/C++ style syntax.


4.2 For Loops

For-each loops:
```
for var in Iterable {
    ...
}
```

I think that allowing Java/C++ style for loops, which look like
` for(int i = 0; i < length, i = i + 1) { ... }`, would be confusing given the
pythonic syntax of for-each loops. They may also not come up that often, since
most of our iteration will probably be on existing graph datastructures, and 
the same functionality can be accomplished with while loops where necessary.


5. Data Types
5.1 Integers
Represents values between −2^63 to 2^63 − 1. Operations: +, -, *, /, %, ^ 
(where / is integer division, like in python 2), <, >, <=, =>, ==, !=

In order of precedence: (*, /, %), (+, -)
                        (<, >, <=, =>), (==, !=)  // doesn't matter

5.2 Booleans
Since our typing system resembles Java or C++, to avoid confusion, perhaps
our boolean literals should as well:

```
bool b1 = true;
bool b2 = false;
```

Boolean binary operators: ==, !=, &&, ||

Boolean unary operator: !

Precedence: !, (==, !=), (&&, ||)

a == b && c => (a == b) && c vs. a == (b && c), which are clearly not equivalent
(the former can only be true if c is true).


5.3 Tuples

Initialization:
```
t = (p1, p2)
```

Access:
```
t.0
t.1
```

(Notes on hardware implementation: could (a) use registers, (b) use vectors
plus splicing whenever we retrieve an element from a list, if bitwidths vary;
could also use splicers with registers to reduce the total # of registers via
register sharing)


5.4 Records

Initialization:
```
r = {f1 = e2; f2 = e2; ...; fn = en}
```

Initialization from another record:

Option 1:
```
r2 = {r with f1 = e1'; ...; fn = en'}
```

Option 2:
```
r2 = {f1 = e1'; ..r}
```

We don't need both initialization types, although supporting both of these 
syntaxes seems perfectly viable; I lean towards the first option because it
looks more like regular english and is perhaps more intuitive to people who
aren't experts in CS, but both seem fine.


5.3 Arrays

Declarations: 

```
int[] arr = Array[n]();     // An array of length n
int[] arr = Array[n](0);    // Initialize the array with 0s
int[] arr = [0, 1, 2, 4];   // Define each element of the array
(node*int)[] depths = Array[node_count()](); // An array of tuples mapping nodes to integers
```

Access/Assignment:
``` arr[i] = e ```

Length:
```arr.length()```

Or, if we're feeling spicy, we could use ```arr.size()``` so that users only
have to remember one keyword (vs. different keywords for arrays and sets/dicts/lists)

6. Functions

Function declaration:

``` t fun(t1 input1, t2 input2, ..., tn inputn) {
    ...
    [return e1, ..., en] // Optional return statement
}
```