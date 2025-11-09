<?php

// Test 1: Interface with templates, class substitutes correctly (OK)
// PHP: No error
/**
 * @template K
 * @template V
 */
interface Collection {
    /**
     * @param K $key
     * @return V|null
     */
    public function get(mixed $key): mixed;
}

/**
 * @implements Collection<string, User>
 */
class UserCollection implements Collection {
    // OK: Substituted types should match
    public function get(mixed $key): mixed {
        return new User();
    }
}

// Test 2: Template substitution with narrowing (depends on implementation)
// PHP: Type hints use mixed, but doc blocks have templates
/**
 * @template T
 */
interface Repository {
    /**
     * @param T $entity
     */
    public function save(mixed $entity): void;

    /**
     * @return T|null
     */
    public function find(int $id): mixed;
}

/**
 * @implements Repository<User>
 */
class UserRepository implements Repository {
    // Types in signatures must still use mixed
    public function save(mixed $entity): void {}

    public function find(int $id): mixed {
        return null;
    }
}

// Test 3: Nested template substitution
// PHP: Doc blocks only, runtime types are mixed
/**
 * @template T
 */
interface Container {
    /**
     * @return T
     */
    public function value(): mixed;
}

/**
 * @template U
 * @implements Container<array<U>>
 */
interface ListContainer extends Container {
    /**
     * @return array<U>
     */
    public function value(): mixed;
}

/**
 * @implements ListContainer<string>
 */
class StringList implements ListContainer {
    public function value(): mixed {
        return [];
    }
}

// Test 4: Template with constraints
// PHP: Constraints in doc blocks
/**
 * @template T of object
 */
interface EntityManager {
    /**
     * @param T $entity
     */
    public function persist(object $entity): void;

    /**
     * @param class-string<T> $class
     * @return T|null
     */
    public function find(string $class, int $id): ?object;
}

/**
 * @implements EntityManager<User>
 */
class UserManager implements EntityManager {
    public function persist(object $entity): void {}

    public function find(string $class, int $id): ?object {
        return null;
    }
}

// Test 5: Multiple template parameters with substitution
// PHP: Doc blocks provide type info
/**
 * @template TKey of array-key
 * @template TValue
 * @template TReturn
 */
interface Transformer {
    /**
     * @param array<TKey, TValue> $input
     * @return array<TKey, TReturn>
     */
    public function transform(array $input): array;
}

/**
 * @implements Transformer<int, string, User>
 */
class StringToUserTransformer implements Transformer {
    public function transform(array $input): array {
        return [];
    }
}

// Dummy class
class User {
    public string $name = "test";
}
