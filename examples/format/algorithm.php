<?php // trail

declare(strict_types=1);

namespace Psl\Password;

use Psl\Default\DefaultInterface;

use const PASSWORD_ARGON2I;
use const PASSWORD_ARGON2ID;
use const PASSWORD_BCRYPT;
use const PASSWORD_DEFAULT;

/**
 * Enumerates supported hashing algorithms for passwords.
 *
 * This enum provides a type-safe way to specify the algorithm to be used for hashing passwords.
 *
 * It includes support for widely used algorithms like Bcrypt, Argon2i,  and Argon2id,
 * and allows for the default algorithm to be used, which is subject to change with
 * future PHP versions to ensure the use of strong, up-to-date cryptographic standards.
 */
enum Algorithm: string implements DefaultInterface
{
    /**
     * The `Algorithm::Argon2i` is used to create new password hashes using the argon2i algorithm.
     *
     * Supported options:
     *      - `memory_cost` ( integer ): Maximum memory (in bytes) that may be used to compute the Argon2 hash.
     *      - `time_cost` ( integer ): Maximum amount of time it may take to compute the Argon2 hash.
     *      - `threads ` ( integer ): Number of threads to use for computing the Argon2 hash
     */
    case Argon2i = "argon2i";

    /**
     * The `Algorithm::Argon2id` is used to create new password hashes using the argon2id algorithm.
     *
     * Supported options:
     *      - `memory_cost` ( integer ): Maximum memory (in bytes) that may be used to compute the Argon2 hash.
     *      - `time_cost` ( integer ): Maximum amount of time it may take to compute the Argon2 hash.
     *      - `threads ` ( integer ): Number of threads to use for computing the Argon2 hash
     */
    case Argon2id = "argon2id";

    /**
     * The `Algorithm::Bcrypt` is used to create new password hashes
     * using the blowfish algorithm.
     *
     * This will result in a hash using the `"$2y$"` crypt format
     * which is always 60 characters wide.
     *
     * Supported options:
     *      - `cost` ( integer ): which denotes the algorithmic cost that should be used.
     */
    case Bcrypt = "bcrypt";

    /**
     * The default algorithm to use for hashing if no algorithm is provided.
     *
     * This may change in newer PHP releases when newer, stronger hashing algorithms are supported.
     *
     * It is worth noting that over time the algorithm behind this case can (and likely will) change.
     *
     * Therefore you should be aware that the length of the resulting hash can change.
     *
     * Therefore, if you use `Psl\Password\Algorithm::Default` you should store
     * the resulting hash in a way that can store more than 60 characters (255 is the recommended width).
     */
    case Default = "default";

    /**
     * Retrieves the PHP built-in constant value associated with each algorithm.
     *
     * This method maps the enum cases to their corresponding PHP built-in constant
     * values used by the password hashing API. It enables seamless integration between
     * the type-safe enum approach and PHP's underlying password hashing mechanism.
     *
     * @psalm-mutation-free
     */
    public function getBuiltinConstantValue(): string
    {
        return match ($this) {
            // Default algo
            static::Default => PASSWORD_DEFAULT,
            // Bcrypt algo
            static::Bcrypt => PASSWORD_BCRYPT,
            // Argon2i algo
            static::Argon2i => PASSWORD_ARGON2I,
            // Argon2id algo ( stronger than Argon2i )
            static::Argon2id => PASSWORD_ARGON2ID,
        };
    }

    /**
     * Provides the default algorithm to use for hashing if no algorithm is provided.
     *
     * It is recommended to store the resulting hash in a way that can accommodate more than 60 characters,
     * as the default algorithm may change in newer PHP releases to incorporate stronger hashing algorithms,
     * potentially leading to longer hash strings.
     *
     * @return static The default algorithm instance, subject to change in future implementations.
     *
     * @pure
     */
    public static function default(): static
    {
        return self::Default /**
             * Trailing multi line
             */;
    }
}
