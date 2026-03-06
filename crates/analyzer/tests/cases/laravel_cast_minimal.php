<?php

declare(strict_types=1);

// Minimal Laravel stubs — just enough class hierarchy for the plugin to recognize
// Eloquent Model subclasses and resolve cast properties.

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
}

namespace Illuminate\Support {
    class Carbon extends \DateTimeImmutable
    {
    }
}

// ─── Application model ──────────────────────────────────────────────────────

namespace App\Models {

    use Illuminate\Database\Eloquent\Model;

    class CastModel extends Model
    {
        protected array $casts = [
            'is_admin'     => 'boolean',
            'age'          => 'integer',
            'score'        => 'float',
            'bio'          => 'string',
            'published_at' => 'datetime',
        ];

        protected array $fillable = ['slug'];

        protected array $attributes = [
            'status' => 'draft',
        ];

        protected array $hidden = ['secret'];
    }
}

// ─── Tests ──────────────────────────────────────────────────────────────────
// The Laravel plugin's ExpressionHook should intercept property access on Model
// subclasses and provide precise types from $casts, $attributes, $fillable, etc.
//
// If the hook is NOT working, the property access falls through to __get which
// returns `mixed`, producing `non-documented-property` and `mixed-assignment`.
//
// We annotate each access with what we expect:
//   - If the plugin resolves a concrete type → no issue expected
//   - If the plugin resolves `mixed` (e.g. $fillable columns) → mixed-assignment
//   - If neither the plugin nor any other mechanism resolves it → non-documented-property + mixed-assignment

namespace Tests\Laravel\CastMinimal {

    use App\Models\CastModel;

    function test_cast_boolean(CastModel $m): void
    {
        // $casts['is_admin'] => 'boolean' → bool
        // The plugin resolves this to bool via resolve_cast_property_type.
        $isAdmin = $m->is_admin;
    }

    function test_cast_integer(CastModel $m): void
    {
        // $casts['age'] => 'integer' → int
        $age = $m->age;
    }

    function test_cast_float(CastModel $m): void
    {
        // $casts['score'] => 'float' → float
        $score = $m->score;
    }

    function test_cast_string(CastModel $m): void
    {
        // $casts['bio'] => 'string' → string
        $bio = $m->bio;
    }

    function test_cast_datetime(CastModel $m): void
    {
        // $casts['published_at'] => 'datetime' → Illuminate\Support\Carbon
        $publishedAt = $m->published_at;
    }

    function test_fillable_column(CastModel $m): void
    {
        // $fillable columns resolve to `mixed` even when the plugin works.
        // @mago-expect analysis:mixed-assignment
        $slug = $m->slug;
    }

    function test_attribute_default(CastModel $m): void
    {
        // $attributes['status'] => 'draft' → literal-string type
        // The plugin resolves this via resolve_attribute_default_type.
        $status = $m->status;
    }

    function test_hidden_column(CastModel $m): void
    {
        // $hidden columns resolve to `mixed` even when the plugin works.
        // @mago-expect analysis:mixed-assignment
        $secret = $m->secret;
    }

    function test_real_property(CastModel $m): void
    {
        // Real declared properties should work normally — no issue expected.
        $exists = $m->exists;
    }

    function test_unknown_property(CastModel $m): void
    {
        // A property not in casts, fillable, hidden, attributes, or declared.
        // The issue filter suppresses non-documented-property on Model subclasses,
        // but __get still returns mixed → mixed-assignment.
        // @mago-expect analysis:mixed-assignment
        $unknown = $m->totally_unknown;
    }
}