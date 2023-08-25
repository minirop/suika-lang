# Suika Lang

Convert your `.ptq` files to `.txt`

## Usage

```console
$ ./suika-lang [-i <input directory>] [-o <output directory>]
```

By default, input and output directories are the current directory.

## Syntax

```js
bg("#000000", 1.0)

if $time_of_day == 0 {
	say("Midori", "Yawn~")
} elseif $time_of_day == 1 {
	say("Midori", "001.ogg", "Everybody left...")
} else {
	say("Everyone is sleeping!")
}

using("increment.txt")

choose(LAKE, "Go to the lake", TOWN, "Go to town", STAY, "Stay here")

label(LAKE)
load("lake.txt")
label(TOWN)
load("town.txt")
label(STAY)
load("nest.txt")
```

For now, everything is a function (will/might change later, especially labels).

There is a special function called `include` which will copy/paste its parameter instead of using `using`. To not include this file in the output, use another extension, like `.inc`, since this software only parses `.ptq` files.
