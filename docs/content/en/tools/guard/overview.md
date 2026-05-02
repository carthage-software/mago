+++
title = "Guard"
description = "What the guard does and how its two halves divide the work."
nav_order = 10
nav_section = "Tools"
nav_subsection = "Guard"
+++
# Guard

`mago guard` enforces architectural boundaries and structural conventions across a PHP project. It covers the same ground as deptrac and arkitect, in a single binary running off Mago's parser.

The tool has two halves: the perimeter guard validates dependencies between layers, and the structural guard enforces conventions on the symbols themselves.

## Perimeter guard

The perimeter guard validates dependency edges. It ensures that different parts of an application only talk to each other in ways you have explicitly allowed, so the domain stays free of infrastructure leaks and the UI cannot reach past the application layer.

Typical rules:

- The `Domain` layer must not depend on any other layer.
- The `UI` layer can depend on `Application` but not the other way around.
- A specific module is only allowed to use an approved list of libraries.

## Structural guard

The structural guard enforces conventions on the symbols themselves: their names, modifiers, supertypes, attributes, and the shape of their containing namespace.

Typical rules:

- All classes in `App\Http\Controllers` must be `final` and end in `Controller`.
- Interfaces under `Domain` must end with `Interface`.
- A specific namespace may only contain `enum` definitions.

## Where to next

- [Usage](/tools/guard/usage/): the common commands and what their output looks like.
- [Command reference](/tools/guard/command-reference/): every flag `mago guard` accepts.
- [Configuration reference](/tools/guard/configuration-reference/): every option Mago accepts under `[guard]`.
