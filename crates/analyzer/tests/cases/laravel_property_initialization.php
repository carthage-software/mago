<?php

declare(strict_types=1);

// ─── Laravel stub classes ────────────────────────────────────────────────────
// Minimal stubs so the analyzer sees the class hierarchy and conventions.
// We keep stubs as simple as possible to avoid unrelated diagnostics.

namespace Illuminate\Database\Eloquent {

    abstract class Model
    {
        /** @var array<string, string> */
        protected array $casts = [];

        /** @var array<string, mixed> */
        protected array $attributes = [];

        /** @var list<string> */
        protected array $fillable = [];

        /** @var list<string> */
        protected array $guarded = ['*'];

        /** @var list<string> */
        protected array $hidden = [];

        protected string $table = '';
        protected string $primaryKey = 'id';
        protected string $keyType = 'int';
        public bool $incrementing = true;
        public bool $exists = false;
        public bool $wasRecentlyCreated = false;
        protected bool $timestamps = true;
        protected string $dateFormat = '';
        protected string $connection = '';
        /** @var list<string> */
        protected array $with = [];
        /** @var list<string> */
        protected array $withCount = [];
        protected int $perPage = 15;
        /** @var list<string> */
        protected array $appends = [];
        /** @var list<string> */
        protected array $visible = [];
        /** @var list<string> */
        protected array $touches = [];
        /** @var list<string> */
        protected array $observables = [];
        /** @var array<string, string> */
        protected array $relations = [];
        /** @var list<string> */
        protected array $dates = [];
        /** @var list<string> */
        protected array $dispatchesEvents = [];

        public function __construct(array $_attributes = [])
        {
        }

        public function __get(string $_name): mixed
        {
            return null;
        }

        public function __set(string $_name, mixed $_value): void
        {
        }

        public function __call(string $_method, array $_parameters): mixed
        {
            return null;
        }

        public static function __callStatic(string $_method, array $_parameters): mixed
        {
            return null;
        }
    }

    // Minimal Builder stub — only needs hierarchy, no methods or typed properties.
    // The property init plugin marks well-known Builder properties ($model, $query,
    // $eagerLoad) as initialized, so we don't need to give them defaults here.
    class Builder
    {
        protected mixed $model = null;
        protected mixed $query = null;
        /** @var array<string, bool> */
        protected array $eagerLoad = [];

        public function __construct()
        {
        }
    }
}

namespace Illuminate\Database\Eloquent\Factories {

    use Illuminate\Database\Eloquent\Model;

    // Minimal Factory stub — only needs hierarchy and well-known properties.
    // Give $model a default to avoid missing-constructor issues.
    abstract class Factory
    {
        protected string $model = '';

        protected ?int $count = null;

        /** @var list<callable> */
        protected array $states = [];

        /** @var list<callable> */
        protected array $afterMaking = [];

        /** @var list<callable> */
        protected array $afterCreating = [];
    }
}

// ─── Application classes ─────────────────────────────────────────────────────

namespace App\Models {

    use Illuminate\Database\Eloquent\Model;

    /**
     * A model that overrides well-known framework properties.
     * These should NOT trigger uninitialized-property warnings because the
     * Laravel PropertyInitializationProvider marks them as initialized.
     */
    class User extends Model
    {
        protected array $fillable = ['name', 'email'];
        protected array $hidden = ['password'];
        protected array $casts = [
            'email_verified_at' => 'datetime',
        ];
    }

    /**
     * A model with typed properties that have no defaults.
     * These represent database-backed columns hydrated at runtime.
     * The PropertyInitializationProvider should treat them as initialized.
     */
    class Article extends Model
    {
        protected string $title;
        protected int $view_count;
        protected ?string $subtitle;
    }

    /**
     * A model that mixes framework properties and custom typed properties.
     */
    class Product extends Model
    {
        protected array $fillable = ['name', 'price'];
        protected array $casts = [
            'price' => 'float',
            'in_stock' => 'boolean',
        ];

        // Typed column properties — should be treated as initialized
        protected string $sku;
        protected float $weight;
    }

    /**
     * A model with static and readonly properties.
     * Static properties should NOT be treated as initialized by the plugin.
     * Readonly properties should NOT be treated as initialized by the plugin.
     */
    class Config extends Model
    {
        // Static typed property without default — the plugin does NOT mark
        // this as initialized (it's not a database column).
        // The analyzer does not report uninitialized-property for static
        // properties, so no issue is expected here.
        protected static string $globalSetting;

        // Readonly typed property without default — also not treated as a
        // database column by the plugin.  The inherited Model constructor
        // satisfies the constructor check, so no issue is reported.
        public readonly string $immutableValue;

        // Regular typed column — should be treated as initialized
        protected string $key;
        protected string $value;
    }
}

namespace App\Builders {

    use Illuminate\Database\Eloquent\Builder;

    /**
     * A custom builder that extends Eloquent Builder.
     * Its well-known inherited properties should be marked as initialized.
     */
    class UserBuilder extends Builder
    {
    }
}

namespace Database\Factories {

    use Illuminate\Database\Eloquent\Factories\Factory;

    /**
     * A factory subclass.
     * Well-known factory properties ($model, $count, etc.) should be
     * marked as initialized.
     */
    class UserFactory extends Factory
    {
    }
}