# Lindenmayer Systems

A (re)exploration of Lindenmayer Systems.

- [Lindenmayer Systems](#lindenmayer-systems)
- [L-System Rule Parsing](#l-system-rule-parsing)
  - [Basic Usage](#basic-usage)
  - [Stochastic Grammars](#stochastic-grammars)
  - [Context Sensitive Grammars](#context-sensitive-grammars)
  - [All of the Above](#all-of-the-above)
  - [Why not Parametric?](#why-not-parametric)
- [L-String Interpretation](#l-string-interpretation)
- [Line Simplification and Joining](#line-simplification-and-joining)
- [3D Perspective Tweaking](#3d-perspective-tweaking)
- [SVG Generation](#svg-generation)
- [SVG Transformations](#svg-transformations)
- [Pen Plotting](#pen-plotting)

I worked on a [class project](https://github.com/macattackftw/fractal_trees) to implement 3D context-free Lindenmayer systems in graduate school.
This is an attempt on doing the same, but for additional grammar types, and with the intent to build a modular transformation pipeline, with the last stage being the generation of an SVG (even for 3D fractals) suitable for for drawing with my [AxiDraw](https://axidraw.com/)

# L-System Rule Parsing

In [*The Algorithmic Beauty of Plants*](http://algorithmicbotany.org/papers/#abop), Lindenmayer and Prusinkiewicz outlined several types of grammars that could be interpreted as algorithmic models of plans.
These grammars are

1. Context-free grammars
2. Stochastic grammars
3. Context-sensitive grammars
4. Parametric grammars

This project implements a [parser](tools/parser.py) for the first three kinds of grammars.

## Basic Usage

Tokens can be multiple characters, and therefore must be comma-separated.
The rules and axiom are white-space insensitive.

**TODO:** Support whitespace-separated tokens.

```shell
$ tools/parser.py --rule 'a -> a, b' --rule 'b -> a' --axiom=a --iterations=3
abaab
$ tools/parser.py --rule 'a -> ab' --rule 'b -> a' --axiom=a --iterations=30
ab
$ tools/parser.py --rule 'a -> ab' --rule 'ab -> a, a' --axiom=a --iterations=3
abab
```

## Stochastic Grammars

If more than one production rule is given for a single token, the first rule given will be chosen.

```shell
$ tools/parser.py --rule 'a -> a' --rule 'a -> b' --axiom='a,a' --iterations=100
aa
```

Probabilities can be specified like so:

```shell
$ tools/parser.py --rule 'a : 0.5 -> a' --rule 'a : 0.5 -> b' --axiom='a,a' --iterations=1 --log-level INFO
2020-08-30 11:36:24,129 - lsystem.grammar - INFO - Using random seed: 4162256033
aa
$ tools/parser.py --rule 'a : 0.5 -> a' --rule 'a : 0.5 -> b' --axiom='a,a' --iterations=1 --log-level INFO
2020-08-30 11:36:26,368 - lsystem.grammar - INFO - Using random seed: 635680691
ba
$ tools/parser.py --rule 'a : 0.5 -> a' --rule 'a : 0.5 -> b' --axiom='a,a' --iterations=1 --log-level INFO
2020-08-30 11:36:28,439 - lsystem.grammar - INFO - Using random seed: 2707414783
bb
```

A random seed may be given via `--seed`.

## Context Sensitive Grammars

One token of left or right (or both) context may be specified.

```shell
$ tools/parser.py --rule 'a>b -> c' --axiom='a,b' --iterations=1
cb
$ tools/parser.py --rule 'b<a -> c' --axiom='b,a' --iterations=1
bc
$ tools/parser.py --rule 'b<a>b -> c' --axiom='b,a,b' --iterations=1
bcb
```

Note that tokens without any matching rules are simply passed-through.

You can also specify a list of tokens to ignore when considering context.

```shell
$ tools/parser.py --rule 'b<a>b -> c' --rule='#ignore:a' --axiom='b,a,a,b' --iterations=1
bccb
```

## All of the Above

See `tools/parser.py --help`. You can mix and match stochastic, context-sensitive, and context-free rules.
It's also possible to pass a JSON config file to avoid incredibly long and hard-to-remember commandline invocations.

However, be aware I've made no attempt at reconcilling any cases where rules don't make sense.
If rules are poorly-formatted, expect an exception.
If probabilities don't sum to 1, expect an exeption.

## Why not Parametric?

Because I'm writing this for fun.

# L-String Interpretation

**TODO:** Given a string of text, interpret it as a sequence of 3D turtle commands.

# Line Simplification and Joining

**TODO:** Turn a collection of potentially overlapping (and containing duplicates) line segments, and turn it into a simplified collection of polygonal lines.

# 3D Perspective Tweaking

**TODO:** View the collection of segments (or polygonal lines) in a 3D viewer that allows for modifying the camera perspective.

# SVG Generation

**TODO:** Simple for the 2D case, but 3D isn't so trivial.

# SVG Transformations

**TODO:**

# Pen Plotting

**TODO:**
