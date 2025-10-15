--- 
title: Architectural Guard
outline: deep
---

# Architectural Guard 🛡️

`mago guard` is a powerful architectural validation utility for enforcing boundaries and coding standards within a PHP project. It acts as a high-performance, single-binary replacement for tools like **deptrac** (for dependency validation) and **arkitect** (for structural rule enforcement).

Think of it as a two-part fortress for your codebase: the **Perimeter Guard** and the **Structural Guard**.

### Perimeter Guard: Defending Your Boundaries

The Perimeter Guard is responsible for **dependency validation**. It ensures that different parts of your application only communicate in ways you've explicitly allowed. Its primary goal is to protect your architectural layers and prevent invalid dependencies from crossing into your core domain logic.

For example, you can enforce rules like:
-   Your `Domain` layer must not depend on any other layer.
-   Your `UI` layer can depend on the `Application` layer, but not the other way around.
-   A specific module is only allowed to use certain approved libraries.

This is crucial for maintaining a clean, decoupled architecture that is easy to maintain and scale.

### Structural Guard: Enforcing Internal Order

The Structural Guard is responsible for **enforcing coding conventions** on the symbols themselves. It inspects the internal structure of your code to ensure it follows a consistent and predictable pattern.

For example, you can enforce rules like:
-   All classes in the `App\Http\Controllers` namespace must be `final` and have names ending in `Controller`.
-   Interfaces in the `Domain` must be named with an `Interface` suffix.
-   A specific namespace may only contain `enum` definitions.

This helps maintain a high level of code quality and consistency across the entire project.

## Dive In

- **[Configuration Reference](./configuration-reference.md)**: A detailed guide to all `guard` settings in `mago.toml`.
- **[Command Reference](./command-reference.md)**: Learn how to run the guard from the command line.

