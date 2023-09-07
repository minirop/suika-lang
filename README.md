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

$GLOBAL = 42

if $time_of_day == 0 {
	say("Midori", "Yawn~")
} elseif $time_of_day == 1 {
	say("Midori", "001.ogg", "Everybody left...")
} else {
	say("Everyone is sleeping!")
}

%name = "john"

switch %name {
	"john" => {
		say("John was here!")
	}
	"carl" => {
		say("John wasn't here.")
	}
	_ => {
		say("who?")
	}
}

using("increment.txt")

choose("Go to the lake" => LAKE, "Go to town" => TOWN, "Stay here" => STAY)

label(LAKE)
load("lake.txt")
label(TOWN)
load("town.txt")
label(STAY)
load("nest.txt")
```

For now, everything is a function (will/might change later, especially labels).

There is a special function called `include` which will copy/paste its parameter instead of using `using`. To not include this file in the output, use another extension, like `.inc`, since this software only parses `.ptq` files.

### Variables

- String variables start with `%` and only 26 are available.
- Local integer variables start with `$` followed by a lowercase letter, and 10000 are available.
- Global integer variables start with `$` followed by an uppercase letter, and 1000 are available.

Global variables are, as their name implies, global across all saves. Local ones are per save.
